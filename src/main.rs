use crate::helper::data;
use log::info;
use mongodb::options::Credential;
use mongodb::{options::ClientOptions, Client, Collection};
use poise::serenity_prelude as serenity;
use std::env;

mod flavors;
mod helper;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {
    collection: Collection<data::User>,
    lastfm_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let discord_token =
        env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN environment variable.");
    let mongo_addr = env::var("MONGO_ADDR").expect("Missing MONGO_ADDR environment variable.");
    let mongo_user = env::var("MONGO_USER").expect("Missing MONGO_USER environment variable.");
    let mongo_pass = env::var("MONGO_PASS").expect("Missing MONGO_PASS environment variable.");
    let lastfm_key = env::var("LASTFM_KEY").expect("Missing LASTFM_KEY environment variable.");

    let mut client_options = ClientOptions::parse(format!("mongodb://{}", mongo_addr)).await?;
    client_options.app_name = Some(String::from("cocoa"));
    client_options.credential = Some(
        Credential::builder()
            .username(mongo_user)
            .password(mongo_pass)
            .build(),
    );

    let client = Client::with_options(client_options).expect("Client connection failed.");

    let db = client.database("cocoa");
    let collection = db.collection::<data::User>("users");

    let ctx_data = Data {
        collection,
        lastfm_key,
    };

    let options: poise::FrameworkOptions<Data, Error> = poise::FrameworkOptions {
        commands: vec![
            flavors::lastfm::link(),
            flavors::lastfm::profile(),
            flavors::lastfm::nowplaying(),
            flavors::lastfm::recent(),
            flavors::lastfm::unlink(),
            flavors::lastfm::topalbums(),
            flavors::lastfm::topartists(),
            flavors::lastfm::toptracks(),
            flavors::silly::ping(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            ..Default::default()
        },

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
                Ok(ctx_data)
            })
        })
        .options(options)
        .build();

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .await;

    match client?.start().await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::from(e.to_string())),
    }
}
