use crate::{Context, Error};
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Colour, CreateEmbedAuthor, CreateEmbedFooter};
use reqwest;
use serde_json::Value;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "profile",
        "link",
        "nowplaying",
        "topartists",
        "toptracks",
        "topalbums",
        "recent"
    )
)]
pub async fn lastfm(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn profile(ctx: Context<'_>) -> Result<(), Error> {
    let id = ctx.author().id.get().to_string();
    let user_rec = sqlx::query!("SELECT username FROM lastfm WHERE user_id = ?", id)
        .fetch_one(&ctx.data().database)
        .await?;
    let user_str = user_rec.username.unwrap();

    let value = fetch_endpoint(&ctx.data().lastfm_key, "user.getInfo", &user_str)
        .await
        .expect("LastFM request failed.");

    let scrobbles = format!(
        "{} scrobbles",
        value["user"]["playcount"].as_str().ok_or("N/A")?
    );

    let tracks = format!(
        "**{}** tracks",
        value["user"]["track_count"].as_str().ok_or("N/A")?
    );
    let albums = format!(
        "**{}** albums",
        value["user"]["album_count"].as_str().ok_or("N/A")?
    );
    let artists = format!(
        "**{}** artists",
        value["user"]["artist_count"].as_str().ok_or("N/A")?
    );

    let count_field = format!("{}\n{}\n{}", tracks, albums, artists);

    let registration_unix = value["user"]["registered"]["unixtime"]
        .as_str()
        .ok_or("0")?
        .parse::<i64>()?;
    let registration_ndt: DateTime<Utc> = DateTime::from_timestamp(registration_unix, 0).unwrap();
    let registration_ago = timeago::Formatter::new().convert_chrono(registration_ndt, Utc::now());

    println!("{}\n{}\n{}", value, registration_unix, registration_ndt);
    let registration_fmt = format!(
        "{}\n{}",
        registration_ago,
        registration_ndt.format("%Y/%m/%d")
    );

    let country = value["user"]["country"].as_str().ok_or("N/A")?;
    let profile_pic = value["user"]["image"][2]["#text"].as_str().ok_or("")?;

    let embed = serenity::CreateEmbed::new()
        .color(Colour::from_rgb(255, 166, 248))
        .thumbnail(profile_pic)
        .title(user_str)
        .field(scrobbles, count_field, true)
        .field("registered", registration_fmt, true)
        .field("country", country, true);

    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await
        .expect("Message reply failed.");
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn link(
    ctx: Context<'_>,
    #[description = "Profile to link"] user: String,
) -> Result<(), Error> {
    fetch_endpoint(&ctx.data().lastfm_key, "user.getInfo", &user)
        .await
        .expect("User doesn't exist.");

    let uid = &ctx.author().id.to_string();
    match sqlx::query!(
        "INSERT INTO lastfm (user_id, username) VALUES (?, ?)",
        uid,
        user
    )
    .execute(&ctx.data().database)
    .await
    {
        Ok(_) => {
            ctx.reply("Success! LastFM linked.").await?;
            Ok(())
        }
        Err(_) => {
            return Err(Error::from(
                "you already have an account attached (or something else broke)",
            ))
        }
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn nowplaying(ctx: Context<'_>) -> Result<(), Error> {
    let id = ctx.author().id.get().to_string();
    let user_rec = sqlx::query!("SELECT username FROM lastfm WHERE user_id = ?", id)
        .fetch_one(&ctx.data().database)
        .await?;
    let user_str = user_rec.username.unwrap();

    let track_coarse = fetch_endpoint(
        &ctx.data().lastfm_key,
        "user.getRecentTracks&limit=1",
        &user_str,
    )
    .await?;

    let artist_name = track_coarse["recenttracks"]["track"][0]["artist"]["#text"]
        .as_str()
        .expect("artist not found...?")
        .to_string();
    let track_name = track_coarse["recenttracks"]["track"][0]["name"]
        .as_str()
        .expect("track not found...?")
        .to_string();

    let method = format!("track.getInfo&artist={}&track={}", artist_name, track_name);
    let track_detailed = fetch_endpoint(&ctx.data().lastfm_key, method.as_str(), &user_str).await?;

    println!("{}", track_detailed);
    let length = track_detailed["track"]["duration"]
        .as_str()
        .ok_or("0")?
        .parse::<i64>()?;

    let album_name = track_detailed["track"]["album"]["title"]
        .as_str()
        .ok_or("0")?;

    let mut tags_map = track_detailed["track"]["toptags"]["tag"]
        .as_array()
        .expect("broken JSON deserialization")
        .iter()
        .map(|tag| tag["name"].as_str().expect("failed tag mapping"));
    let tags_str = tags_map.collect::<Vec<&str>>().join(", ");

    let play_count = track_detailed["track"]["userplaycount"]
        .as_str()
        .ok_or("0")?
        .parse::<i64>()?;

    let album_cover = track_detailed["track"]["album"]["image"][2]["#text"]
        .as_str()
        .ok_or("")?;

    let title = format!("{} is now playing...", user_str);
    let field_title = format!(
        "{} — {} `[{}:{}]`",
        artist_name,
        track_name,
        length / 60000,
        (length % 60000) / 1000
    );
    let field_desc = format!(":cd: {}\n> {} plays", album_name, play_count);

    let embed = serenity::CreateEmbed::new()
        .color(Colour::from_rgb(255, 166, 248))
        .thumbnail(album_cover)
        .author(CreateEmbedAuthor::new(title))
        .title(field_title)
        .description(field_desc)
        .footer(CreateEmbedFooter::new(tags_str));

    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await
        .expect("Message reply failed.");
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

pub async fn fetch_endpoint(api_key: &str, method: &str, user: &String) -> Result<Value, Error> {
    let user_param_name = match method {
        "user.getInfo" => "user",
        "user.getRecentTracks" => "user",
        "track.getInfo" => "username",
        _ => "user",
    };

    let endpoint = format!(
        "https://ws.audioscrobbler.com/2.0?format=json&api_key={}&method={}&{}={}",
        api_key, method, user_param_name, user
    );

    let json = reqwest::get(endpoint).await?.text().await?;
    let parsed: Value = serde_json::from_str(&json)?;

    if parsed["error"].is_null() {
        Ok(parsed)
    } else {
        Err(Error::from(""))
    }
}
