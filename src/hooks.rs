use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandError, DispatchError},
    model::channel::Message,
};
use tracing::{event, span, Instrument, Level};

use crate::sunny_log;
use crate::utils::SunnyError;
use std::env;

#[hook]
pub async fn before_hook(ctx: &Context, msg: &Message, _cmd_name: &str) -> bool {
    if let Some(channel) = msg.channel(&ctx.cache).await {
        if let Some(channel_guild) = channel.clone().guild() {
            let channel_name =
                env::var("TEXT_CHANNEL_COMMAND_ACCEPTED").unwrap_or("dwin_audio".to_string());
            if channel_guild.name == channel_name {
                return true;
            }
        }
    }

    let message_reply = env::var("NOT_IN_CHANNEL_MESSAGE_REPLY")
        .unwrap_or("This command is not allowed here".to_string());

    let _message_reply = msg.reply(&ctx.http, message_reply).await;
    return false;
}

#[hook]
pub async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    let span = span!(Level::WARN, "dispatch_error_hook", %msg.content, ?error);
    async move {
        match error {
            DispatchError::CheckFailed(_check, reason) => {
                sunny_log!(&SunnyError::from(reason), ctx, msg, Level::WARN);
            }
            _ => {
                event!(Level::ERROR, ?error, "unknown dispatch error");
            }
        }
    }
    .instrument(span)
    .await
}

#[hook]
pub async fn after_hook(
    ctx: &Context,
    msg: &Message,
    cmd_name: &str,
    error: Result<(), CommandError>,
) {
    let span = span!(Level::WARN, "after_hook", %msg.content, ?cmd_name);
    let _delete_call = msg.delete(&ctx.http).await;
    async move {
        // Print out an error if it happened
        if let Err(why) = error {
            if let Some(reason) = why.downcast_ref::<SunnyError>() {
                sunny_log!(reason, ctx, msg, Level::WARN);
            } else {
                event!(Level::ERROR, %cmd_name, %why, "Unknown error");
            }
        }
    }
    .instrument(span)
    .await
}

// async fn delete_message(ctx: &Context, msg: &Message) {
//     msg.delete(ctx.http);
// }
