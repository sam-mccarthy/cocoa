use std::env;
use serenity::all::standard::macros::group;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::framework::standard::{StandardFramework, Configuration, CommandResult};

mod flavors {
    mod silly;
}

#[group]
#[commands(ping)]
struct Silly;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .group(&SILLY_GROUP);
    framework.configure(Configuration::new().prefix("-"));

    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}