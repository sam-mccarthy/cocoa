use poise::serenity_prelude::CreateEmbedFooter;

use crate::{Context, Error};
use crate::helper::data::User;
use crate::helper::discord::{cocoa_embed, cocoa_reply_embed, cocoa_reply_str};

#[poise::command(prefix_command, slash_command)]
pub async fn profile(ctx: Context<'_>) -> Result<(), Error> {
    let user_ref = ctx.invocation_data::<User>().await;
    let Some(user) = user_ref.as_deref() else {
        panic!("")
    };

    let embed = cocoa_embed(ctx)
        .await
        .title(ctx.author().tag())
        .field("commands!", user.command_count.to_string(), true)
        .field("pieces!", user.currency.to_string(), true)
        .field("level!", user.experience.to_string(), true)
        .footer(CreateEmbedFooter::new(
            "this is a placeholder command more or less",
        ));

    cocoa_reply_embed(ctx, embed).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn commands(ctx: Context<'_>) -> Result<(), Error> {
    let user_ref = ctx.invocation_data::<User>().await;
    let Some(user) = user_ref.as_deref() else {
        panic!("")
    };

    cocoa_reply_str(ctx, format!("you've used {} commands!", user.command_count)).await?;
    Ok(())
}
