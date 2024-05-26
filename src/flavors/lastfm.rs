use crate::helper::{api, data};
use crate::{Context, Error};

use crate::helper::discord::{cocoa_embed, cocoa_reply_embed, cocoa_reply_str};
use chrono::{DateTime, Utc};
use poise::serenity_prelude::CreateEmbedFooter;

#[poise::command(prefix_command, slash_command)]
pub async fn profile(ctx: Context<'_>) -> Result<(), Error> {
    let id = ctx.author().id.to_string();
    let user_str = data::fetch_lastfm_username(&ctx.data().database, id).await?;

    let value = api::get_from_lastfm(&ctx.data().lastfm_key, "user.getInfo", &user_str).await?;

    let scrobbles = format!(
        "{} scrobbles",
        value["user"]["playcount"].as_str().unwrap_or("N/A")
    );

    let tracks = format!(
        "**{}** tracks",
        value["user"]["track_count"].as_str().unwrap_or("N/A")
    );
    let albums = format!(
        "**{}** albums",
        value["user"]["album_count"].as_str().unwrap_or("N/A")
    );
    let artists = format!(
        "**{}** artists",
        value["user"]["artist_count"].as_str().unwrap_or("N/A")
    );

    let count_field = format!("{}\n{}\n{}", tracks, albums, artists);

    let registration_unix = value["user"]["registered"]["unixtime"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()?;
    let registration_ago = format!("<t:{}:R>", registration_unix);

    let country = value["user"]["country"].as_str().unwrap_or("N/A");
    let profile_pic = value["user"]["image"][2]["#text"].as_str().unwrap_or("");

    let embed = cocoa_embed(ctx)
        .thumbnail(profile_pic)
        .title(user_str)
        .field(scrobbles, count_field, true)
        .field("registered", registration_ago, true)
        .field("country", country, true);

    cocoa_reply_embed(ctx, embed).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn link(
    ctx: Context<'_>,
    #[description = "Profile to link"] user: String,
) -> Result<(), Error> {
    api::get_from_lastfm(&ctx.data().lastfm_key, "user.getInfo", &user).await?;

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
            cocoa_reply_str(ctx, String::from("Success! LastFM linked.")).await?;
            Ok(())
        }
        Err(_) => {
            return Err(Error::from(
                "You already have an account attached (or something else broke).",
            ))
        }
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn unlink(ctx: Context<'_>) -> Result<(), Error> {
    let uid = &ctx.author().id.to_string();
    match sqlx::query!("DELETE FROM lastfm WHERE user_id = ?", uid)
        .execute(&ctx.data().database)
        .await
    {
        Ok(_) => {
            cocoa_reply_str(ctx, String::from("Success! LastFM unlinked.")).await?;
            Ok(())
        }
        Err(_) => return Err(Error::from("You don't have a LastFM account linked.")),
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn nowplaying(ctx: Context<'_>) -> Result<(), Error> {
    let id = ctx.author().id.to_string();
    let user_str = data::fetch_lastfm_username(&ctx.data().database, id).await?;

    let track_coarse = api::get_from_lastfm(
        &ctx.data().lastfm_key,
        "user.getRecentTracks&limit=1",
        &user_str,
    )
    .await?;

    let artist_name = track_coarse["recenttracks"]["track"][0]["artist"]["#text"]
        .as_str()
        .ok_or("Artist not found, somehow.")?
        .to_string();
    let track_name = track_coarse["recenttracks"]["track"][0]["name"]
        .as_str()
        .ok_or("Track not found, somehow.")?
        .to_string();

    let method = format!("track.getInfo&artist={}&track={}", artist_name, track_name);
    let track_detailed =
        api::get_from_lastfm(&ctx.data().lastfm_key, method.as_str(), &user_str).await?;

    let length = track_detailed["track"]["duration"]
        .as_str()
        .unwrap_or("N/A")
        .parse::<i64>()?;

    let album_name = track_detailed["track"]["album"]["title"]
        .as_str()
        .unwrap_or("N/A");

    let tags = track_detailed["track"]["toptags"]["tag"]
        .as_array()
        .ok_or("Broken JSON deserialization (nowplaying)")?
        .iter()
        .map(|tag| tag["name"].as_str().unwrap_or("???"))
        .collect::<Vec<&str>>()
        .join(", ");

    let play_count = track_detailed["track"]["userplaycount"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()?;

    let album_cover = track_detailed["track"]["album"]["image"][2]["#text"]
        .as_str()
        .unwrap_or("");

    let title = format!(
        "{} — {} `[{}:{}]`",
        artist_name,
        track_name,
        length / 60000,
        (length % 60000) / 1000
    );
    let description = format!(":cd: {}\n> {} plays", album_name, play_count);

    let embed = cocoa_embed(ctx)
        .thumbnail(album_cover)
        .title(title)
        .description(description)
        .footer(CreateEmbedFooter::new(tags));

    cocoa_reply_embed(ctx, embed).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn recent(ctx: Context<'_>) -> Result<(), Error> {
    let id = ctx.author().id.to_string();
    let user_str = data::fetch_lastfm_username(&ctx.data().database, id).await?;

    let data = api::get_from_lastfm(
        &ctx.data().lastfm_key,
        "user.getRecentTracks&limit=10",
        &user_str,
    )
    .await?;

    let songs = data["recenttracks"]["track"]
        .as_array()
        .ok_or("Failed to parse.")?
        .iter()
        .map(|obj| {
            let artist = obj["artist"]["#text"].as_str().unwrap_or("???");
            let name = obj["name"].as_str().unwrap_or("???");

            let timestamp = obj["date"]["uts"]
                .as_str()
                .unwrap_or("0")
                .parse::<i64>()
                .unwrap();
            let ago = if timestamp != 0 {
                timeago::Formatter::new()
                    .convert_chrono(DateTime::from_timestamp(timestamp, 0).unwrap(), Utc::now())
            } else {
                String::from("Now")
            };

            format!("`{}` **{} — {}**", ago, artist, name)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let album_cover = data["recenttracks"]["track"][0]["image"][2]["#text"]
        .as_str()
        .unwrap_or("");

    let embed = cocoa_embed(ctx).thumbnail(album_cover).description(songs);
    cocoa_reply_embed(ctx, embed).await?;
    Ok(())
}
