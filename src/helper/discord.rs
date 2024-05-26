use crate::Context;
use poise::serenity_prelude as serenity;
use serenity::{Colour, CreateEmbedAuthor};

pub fn cocoa_embed(ctx: Context<'_>) -> serenity::CreateEmbed {
    let author = ctx.author().global_name.as_ref();
    let g = author.unwrap();

    let author_str = format!(
        "{} - [{}]",
        ctx.author()
            .global_name
            .as_ref()
            .unwrap_or(&ctx.author().name),
        ctx.command().name
    );

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
