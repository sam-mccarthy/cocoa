use crate::Context;

use poise::{serenity_prelude as serenity, ReplyHandle};
use serenity::{Colour, CreateEmbedAuthor};

pub async fn cocoa_embed(ctx: Context<'_>) -> serenity::CreateEmbed {
    let author_str = format!("{}!", ctx.command().name);

    serenity::CreateEmbed::new()
        .color(Colour::from_rgb(255, 166, 248))
        .author(
            CreateEmbedAuthor::new(author_str).icon_url(
                ctx.author()
                    .avatar_url()
                    .unwrap_or(ctx.author().default_avatar_url()),
            ),
        )
}

pub async fn cocoa_reply_str(
    ctx: Context<'_>,
    text: String,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    let embed = cocoa_embed(ctx).await.description(text);
    cocoa_reply_embed(ctx, embed).await
}

pub async fn cocoa_reply_embed(
    ctx: Context<'_>,
    embed: serenity::CreateEmbed,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await
}
