use crate::helper::data;
use log::info;
use mongodb::options::Credential;
use mongodb::{options::ClientOptions, Client, Collection};
use poise::serenity_prelude as serenity;
use std::{env, fs};

mod flavors;
mod helper;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {
    collection: Collection<data::User>,
    lastfm_key: String,
}

struct Keys {
    discord_token: String,
    mongo_addr: String,
    mongo_user: String,
    mongo_pass: String,
    lastfm_key: String,
}

fn load_env(key: &str) -> Result<String, Error> {
    let Some(str) = (match env::var(key) {
        Ok(str) => Some(str),
        Err(_) => match env::var(format!("{}_FILE", key)) {
            Ok(path) => fs::read_to_string(path).ok(),
            Err(_) => None,
        },
    }) else {
        panic!("");
    };

    println!("{} > {:?}", str, str.as_bytes());

    Ok(str)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let keys = Keys {
        discord_token: load_env("DISCORD_TOKEN")?,
        mongo_addr: load_env("MONGO_ADDR")?,
        mongo_user: load_env("MONGO_USER")?,
        mongo_pass: load_env("MONGO_PASS")?,
        lastfm_key: load_env("LASTFM_KEY")?,
    };

    let mut client_options = ClientOptions::parse(format!("mongodb://{}", keys.mongo_addr)).await?;
    client_options.app_name = Some(String::from("cocoa"));
    client_options.credential = Some(
        Credential::builder()
            .username(keys.mongo_user)
            .password(keys.mongo_pass)
            .source(String::from("cocoa"))
            .build(),
    );

    let client = Client::with_options(client_options).expect("Client connection failed.");

    let db = client.database("cocoa");
    let collection = db.collection::<data::User>("users");

    let ctx_data = Data {
        collection,
        lastfm_key: keys.lastfm_key,
    };

    let options: poise::FrameworkOptions<Data, Error> = poise::FrameworkOptions {
        commands: vec![
            flavors::lastfm::link(),
            flavors::lastfm::profile(),
            flavors::lastfm::nowplaying(),
            flavors::lastfm::recent(),
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

    let client = serenity::ClientBuilder::new(keys.discord_token, intents)
        .framework(framework)
        .await;

    match client?.start().await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::from(e.to_string())),
    }
}
