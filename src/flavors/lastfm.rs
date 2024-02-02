use json;
use json::JsonValue;
use reqwest;

use crate::{Context, Error};

#[poise::command(prefix_command, slash_command, subcommands("link", "nowplaying", "topartists", "toptracks", "topalbums", "recent"))]
pub async fn lastfm(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn profile(ctx: Context<'_>) -> Result<(), Error> {
    let user = sqlx::query!("SELECT username FROM lastfm WHERE user_id = ?", ctx.author().id).fetch_one(&ctx.data().database);
    fetch_endpoint(ctx.data().lastfm_key, "user.getInfo", "");
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn link(ctx: Context<'_>, #[description = "Profile to link"] user: String) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn nowplaying(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn topartists(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn toptracks(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn topalbums(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn recent(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

pub async fn fetch_endpoint(api_key: String, method: String, user: String) -> Result<JsonValue, Error> {
    let endpoint = format!("https://ws.audioscrobbler.com/2.0?method={}&api_key={}&user={}", method, api_key, user);
    let json = reqwest::get(endpoint).await?.text().await?;
    let parsed = json::parse(&json)?;

    if parsed["error"].is_null() {
        Ok(parsed)
    } else {
        Err(Box::new("Failed get request."))
    }
}