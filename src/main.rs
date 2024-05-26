use poise::serenity_prelude as serenity;
use std::env;

mod flavors;
mod helper;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {
    database: sqlx::SqlitePool,
    lastfm_key: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to user database.");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Failed database migration.");

    let lastfm_key = env::var("LASTFM_KEY").expect("Missing LASTFM_KEY environment variable");
    let ctx_data = Data {
        database,
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
                println!(
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

    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN environment variable.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
