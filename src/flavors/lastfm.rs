use crate::helper::{api, data};
use crate::{Context, Error};

use crate::helper::discord::{cocoa_embed, cocoa_reply_embed, cocoa_reply_str};
use chrono::{DateTime, Utc};
use poise::serenity_prelude::CreateEmbedFooter;

#[poise::command(prefix_command, slash_command)]
pub async fn profile(ctx: Context<'_>) -> Result<(), Error> {
    let id = ctx.author().id.to_string();
    let user_str = data::fetch_lastfm_username(&ctx.data().database, id).await?;

    let data = api::get_from_lastfm(&ctx.data().lastfm_key, "user.getInfo", &user_str).await?;

    let scrobbles = format!(
        "{} scrobbles",
        data["user"]["playcount"].as_str().unwrap_or("N/A")
    );

    let tracks = format!(
        "**{}** tracks",
        data["user"]["track_count"].as_str().unwrap_or("N/A")
    );
    let albums = format!(
        "**{}** albums",
        data["user"]["album_count"].as_str().unwrap_or("N/A")
    );
    let artists = format!(
        "**{}** artists",
        data["user"]["artist_count"].as_str().unwrap_or("N/A")
    );

    let count_field = format!("{}\n{}\n{}", tracks, albums, artists);

    let registration_unix = data["user"]["registered"]["unixtime"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()?;
    let registration_ago = format!("<t:{}:R>", registration_unix);

    let country = data["user"]["country"].as_str().unwrap_or("N/A");
    let profile_pic = data["user"]["image"][2]["#text"].as_str().unwrap_or("");

    let embed = cocoa_embed(ctx)
        .await
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

    let title = if length > 0 {
        format!(
            "{} — {} `[{}:{}]`",
            artist_name,
            track_name,
            length / 60000,
            (length % 60000) / 1000
        )
    } else {
        format!("{} — {}", artist_name, track_name,)
    };

    let description = format!(":cd: {}\n> {} plays", album_name, play_count);

    let embed = cocoa_embed(ctx)
        .await
        .thumbnail(album_cover)
        .title(title)
        .description(description)
        .footer(CreateEmbedFooter::new(tags));

    cocoa_reply_embed(ctx, embed).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn recent(ctx: Context<'_>) -> Result<(), Error> {
    list(ctx, "user.getRecentTracks&limit=12", |(_, obj)| {
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
            String::from("a moment ago")
        };

        format!("`{}` **{} — {}**", ago, artist, name)
    })
    .await
}

#[poise::command(prefix_command, slash_command)]
pub async fn topalbums(ctx: Context<'_>) -> Result<(), Error> {
    list(ctx, "user.getTopAlbums&limit=12", |(idx, obj)| {
        let plays = obj["playcount"].as_str().unwrap_or("0");
        let artist = obj["artist"]["name"].as_str().unwrap_or("???");
        let album = obj["name"].as_str().unwrap_or("???");
        format!(
            "`#{: >2}` **{}** plays • **{} — {}**",
            idx + 1,
            plays,
            artist,
            album
        )
    })
    .await
}

#[poise::command(prefix_command, slash_command)]
pub async fn topartists(ctx: Context<'_>) -> Result<(), Error> {
    list(ctx, "user.getTopArtists&limit=12", |(idx, obj)| {
        let plays = obj["playcount"].as_str().unwrap_or("0");
        let artist = obj["name"].as_str().unwrap_or("???");
        format!("`#{: >2}` **{}** plays • **{}**", idx + 1, plays, artist)
    })
    .await
}

#[poise::command(prefix_command, slash_command)]
pub async fn toptracks(ctx: Context<'_>) -> Result<(), Error> {
    list(ctx, "user.getTopTracks&limit=12", |(idx, obj)| {
        let plays = obj["playcount"].as_str().unwrap_or("0");
        let artist = obj["artist"]["name"].as_str().unwrap_or("???");
        let song = obj["name"].as_str().unwrap_or("???");
        format!(
            "`#{: >2}` **{}** plays • **{} — {}**",
            idx + 1,
            plays,
            artist,
            song
        )
    })
    .await
}

pub async fn list(
    ctx: Context<'_>,
    api_method: &str,
    parser: fn((usize, &serde_json::Value)) -> String,
) -> Result<(), Error> {
    let id = ctx.author().id.to_string();
    let user_str = data::fetch_lastfm_username(&ctx.data().database, id).await?;

    let data = api::get_from_lastfm(&ctx.data().lastfm_key, &api_method, &user_str).await?;

    let keys = match api_method.split('&').next().unwrap_or("") {
        "user.getTopTracks" => ("toptracks", "track"),
        "user.getTopArtists" => ("topartists", "artist"),
        "user.getTopAlbums" => ("topalbums", "album"),
        "user.getRecentTracks" => ("recenttracks", "track"),
        _ => panic!("unimplemented!"),
    };

    let list_items = &data[keys.0][keys.1];

    let songs = list_items
        .as_array()
        .ok_or("List of items is not a list.")?
        .iter()
        .enumerate()
        .map(parser)
        .collect::<Vec<String>>()
        .join("\n");

    let album_cover = list_items[0]["image"][2]["#text"].as_str().unwrap_or("");
    let total = format!(
        "total: {}",
        data[keys.0]["@attr"]["total"].as_str().unwrap_or("0")
    );
    let embed = cocoa_embed(ctx)
        .await
        .thumbnail(album_cover)
        .description(songs)
        .footer(CreateEmbedFooter::new(total));
    cocoa_reply_embed(ctx, embed).await?;
    Ok(())
}
