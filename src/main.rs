use std::{env, fs};

use mongodb::Client;
use mongodb::options::{ClientOptions, Credential};

use poise::serenity_prelude as serenity;

use log::{debug, error, info};

use crate::helper::data::{find_user, insert_user, User};
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
    match env::var(key) {
        Ok(str) => Some(str),
        // If the env. var. doesn't exist, check if there's a listed file for it
        Err(_) => match env::var(format!("{}_FILE", key)) {
            Ok(path) => fs::read_to_string(path).ok(),
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

    // Attempt connection, pull db and collection handles
    let client = Client::with_options(client_options).expect("Client connection failed.");

    let db = client.database("cocoa");
    let collection = db.collection::<User>("users");

    let options: poise::FrameworkOptions<mongodb::Collection<User>, Error> = poise::FrameworkOptions {
        commands: vec![
            flavors::silly::ping(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            ..Default::default()
        },
        pre_command: |context| Box::pin(async move {
            // Here, we set the invocation data within the context to the DB's userdata.
            // If no user is find, we insert a new one.
            let collection = context.data();
            let id = context.author().id.get();
            context.set_invocation_data(match find_user(collection, id).await {
                Ok(user) => user,
                Err(_) => insert_user(collection, id).await.expect("Database insertion failed.")
            }).await;

            debug!("User {} ({}): Command {} executed", context.author().tag(), context.author().id, context.command().qualified_name);
        }),
        on_error: |error| Box::pin(async move {
            // We want to handle errors with style (fancy embedding)
            match error.ctx() {
                Some(ctx) => { cocoa_reply_str(ctx, error.to_string()).await.ok(); },
                None => error!("Error caught - missing context: {}", error.to_string())
            };
        }),

        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!(
                    "Logged into {} guilds as {}",
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

    match client?.start().await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::from(e.to_string())),
    }
}
