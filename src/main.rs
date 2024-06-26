use std::{env, fs};

use log::{debug, error, info, trace};
use mongodb::bson::doc;
use mongodb::Client;
use mongodb::options::{ClientOptions, Credential};
use poise::{FrameworkError, serenity_prelude as serenity};

use crate::helper::data::{find_user, insert_user, update_user_inc, User};
use crate::helper::discord::cocoa_reply_str;

mod flavors;
mod helper;

// Error container for ease of error handling
// Context container for simplicity of type
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, mongodb::Collection<User>, Error>;

// Environment variable keys
struct Keys {
    discord_token: String,
    mongo_addr: String,
    mongo_user: String,
    mongo_pass: String,
}

// Loads environment variables - optionally from files
fn load_env(key: &str) -> Result<String, Error> {
    trace!("Loading environment variable {}", key);
    match env::var(key) {
        Ok(str) => {
            debug!("Successfully loaded environment variable {}", key);
            Some(str)
        }
        // If the env. var. doesn't exist, check if there's a listed file for it
        Err(_) => match env::var(format!("{}_FILE", key)) {
            Ok(path) => {
                debug!("Loading {}_FILE from disk", key);
                fs::read_to_string(path).ok()
            }
            Err(_) => None,
        },
    }
    .ok_or(Error::from(format!(
        "Missing environment variable: {}",
        key
    )))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let keys = Keys {
        discord_token: load_env("DISCORD_TOKEN")?,
        mongo_addr: load_env("MONGO_ADDR")?,
        // The implementation of SASLprep is sensitive to whitespace,
        // so we'll trim it here to make things a little less error-prone.
        mongo_user: String::from(load_env("MONGO_USER")?.trim()),
        mongo_pass: String::from(load_env("MONGO_PASS")?.trim()),
    };
    info!("Loaded environment variables");

    // Load credentials for MongoDB into struct
    let mut client_options = ClientOptions::parse(format!("mongodb://{}", keys.mongo_addr)).await?;
    client_options.app_name = Some(String::from("cocoa"));
    client_options.credential = Some(
        Credential::builder()
            .username(keys.mongo_user)
            .password(keys.mongo_pass)
            .source(String::from("cocoa"))
            .build(),
    );

    info!("Connecting to MongoDB");
    // Attempt connection, pull db and collection handles
    let client = Client::with_options(client_options).unwrap();

    debug!("Grabbing database");
    let db = client.database("cocoa");
    debug!("Grabbing collection");
    let collection = db.collection::<User>("users");

    let options: poise::FrameworkOptions<mongodb::Collection<User>, Error> =
        poise::FrameworkOptions {
            commands: vec![
                flavors::silly::ping(),
                flavors::silly::pong(),
                flavors::user::commands(),
                flavors::user::profile(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                // This is only going to temporarily be +, until guild settings are added
                prefix: Some("+".into()),
                ..Default::default()
            },
            pre_command: |context| {
                Box::pin(async move {
                    // Here, we set the invocation data within the context to the DB's userdata.
                    // If no user is find, we insert a new one.
                    let collection = context.data();
                    let tag = context.author().tag();
                    let id = context.author().id.get();

                    context
                        .set_invocation_data(match find_user(collection, id).await {
                            Ok(user) => user,
                            Err(_) => {
                                debug!("Couldn't find user {} ({}), inserting new", tag, id);
                                insert_user(collection, id)
                                    .await
                                    .expect("Database user insertion failed.")
                            }
                        })
                        .await;

                    debug!(
                        "User {} ({}): Command {} executed",
                        context.author().tag(),
                        context.author().id,
                        context.command().qualified_name
                    );
                })
            },
            post_command: |context| {
                Box::pin(async move {
                    // We need to update the running command count for the user.
                    update_user_inc(
                        context.data(),
                        context.author().id.get(),
                        doc! { "command_count": 1},
                    )
                    .await
                    .expect("Database user update failed.");
                })
            },
            on_error: |error| {
                Box::pin(async move {
                    // We want to handle errors with style (fancy embedding)
                    match error {
                        FrameworkError::Command { error, ctx, .. } => {
                            cocoa_reply_str(ctx, error.to_string()).await.ok();
                        }
                        _ => error!("Generic error: {}", error),
                    };
                })
            },

            ..Default::default()
        };

    info!("Setting up framework and registering slash commands");
    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!(
                    "Logged into {} guild(s) as {}",
                    _ready.guilds.len(),
                    _ready.user.name
                );
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(collection)
            })
        })
        .options(options)
        .build();

    // We might need to expand the intents later, but for now this is fine
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(keys.discord_token, intents)
        .framework(framework)
        .await;

    info!("Starting client");
    match client?.start().await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::from(e.to_string())),
    }
}
