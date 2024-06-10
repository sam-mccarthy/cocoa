use crate::{Context, Error};
use crate::helper::discord::cocoa_reply_str;

#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    cocoa_reply_str(ctx, String::from("pong!")).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn pong(ctx: Context<'_>) -> Result<(), Error> {
    cocoa_reply_str(ctx, String::from("ping!")).await?;
    Ok(())
}