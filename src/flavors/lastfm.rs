use crate::helper::api::get_from_lastfm;
use crate::{Context, Error};

use crate::helper::data::fetch_lastfm_username;
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Colour, CreateEmbedAuthor, CreateEmbedFooter};
use reqwest;

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
    let id = ctx.author().id.to_string();
    let user_str = fetch_lastfm_username(&ctx.data().database, id).await?;

    let value = get_from_lastfm(&ctx.data().lastfm_key, "user.getInfo", &user_str)
        .await
        .expect("LastFM request failed.");

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

    let embed = serenity::CreateEmbed::new()
        .color(Colour::from_rgb(255, 166, 248))
        .thumbnail(profile_pic)
        .title(user_str)
        .field(scrobbles, count_field, true)
        .field("registered", registration_ago, true)
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
    get_from_lastfm(&ctx.data().lastfm_key, "user.getInfo", &user)
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
    let id = ctx.author().id.to_string();
    let user_str = fetch_lastfm_username(&ctx.data().database, id).await?;

    let track_coarse = get_from_lastfm(
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
    let track_detailed =
        get_from_lastfm(&ctx.data().lastfm_key, method.as_str(), &user_str).await?;

    let length = track_detailed["track"]["duration"]
        .as_str()
        .unwrap_or("N/A")
        .parse::<i64>()?;

    let album_name = track_detailed["track"]["album"]["title"]
        .as_str()
        .unwrap_or("N/A");

    let tags = track_detailed["track"]["toptags"]["tag"]
        .as_array()
        .expect("Broken JSON deserialization (nowplaying)")
        .iter()
        .map(|tag| tag["name"].as_str().expect("failed tag mapping"))
        .collect::<Vec<&str>>()
        .join(", ");

    let play_count = track_detailed["track"]["userplaycount"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()?;

    let album_cover = track_detailed["track"]["album"]["image"][2]["#text"]
        .as_str()
        .unwrap_or("");

    let author = format!("{} is now playing...", user_str);
    let title = format!(
        "{} — {} `[{}:{}]`",
        artist_name,
        track_name,
        length / 60000,
        (length % 60000) / 1000
    );
    let description = format!(":cd: {}\n> {} plays", album_name, play_count);

    let embed = serenity::CreateEmbed::new()
        .color(Colour::from_rgb(255, 166, 248))
        .thumbnail(album_cover)
        .title(title)
        .description(description)
        .footer(CreateEmbedFooter::new(tags));

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
    let id = ctx.author().id.to_string();
    let user_str = fetch_lastfm_username(&ctx.data().database, id).await?;

    let data = get_from_lastfm(
        &ctx.data().lastfm_key,
        "user.getRecentTracks&limit=10",
        &user_str,
    )
    .await?;

    let songs = data["recenttracks"]["track"]
        .as_array()
        .expect("Broken JSON deserialization (recent)")
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

    let author = format!("{} has listened to...", user_str);
    let album_cover = data["recenttracks"]["track"][0]["image"][2]["#text"]
        .as_str()
        .unwrap_or("");

    let embed = serenity::CreateEmbed::new()
        .color(Colour::from_rgb(255, 166, 248))
        .thumbnail(album_cover)
        .author(CreateEmbedAuthor::new(author))
        .description(songs);

    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await
        .expect("Message reply failed.");
    Ok(())
}
