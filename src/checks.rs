use serenity::{
    client::Context,
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::prelude::*,
    prelude::Mentionable,
};
use tracing::{span, Instrument, Level};

use crate::{
    helper_functions::{
        clear_messages, does_bot_need_to_join, is_channel_dwin_audio_helper, join_voice,
    },
    utils::SunnyError,
};

#[check]
#[name = "In_Voice"]
#[display_in_help]
// Ensures a command is only usable if in the same voice channel as sunny
pub async fn in_same_voice_check(
    ctx: &Context,
    msg: &Message,
    _args: &mut Args,
    _command_options: &CommandOptions,
) -> Result<(), Reason> {
    let span = span!(Level::INFO, "in_same_voice_check", ?msg);
    async move {
        let songbird = songbird::get(ctx)
            .await
            .ok_or_else(|| SunnyError::log("Failed to get songbird"))?;

        let guild_id = msg
            .guild_id
            .ok_or_else(|| SunnyError::log("Guild ID Empty"))?;

        let channel = {
            let songbird_call_m = songbird
                .get(guild_id)
                .ok_or_else(|| SunnyError::user("Not currently in a call"))?;

            let songbird_call = songbird_call_m.lock().await;

            songbird_call
                .current_channel()
                .ok_or_else(|| SunnyError::log("Couldn't find songbird channel"))?
        };

        let name = ChannelId(channel.0);

        let guild = msg
            .guild(&ctx.cache)
            .await
            .ok_or_else(|| SunnyError::log("Couldn't get guild"))?;

        let mut states = guild.voice_states.values();

        states
            .any(|vs| match vs.channel_id {
                Some(c_id) => channel.0 == c_id.0 && vs.user_id.0 == msg.author.id.0,
                None => false,
            })
            .then(|| ())
            .ok_or_else(|| {
                SunnyError::user(
                    format!("I only take requests from users in {}", name.mention()).as_str(),
                )
            })?;
        Ok(())
    }
    .instrument(span)
    .await
}

#[check]
#[name = "Join_Voice"]
#[display_in_help]
// Joins the voice channel of the user that posted the request
pub async fn in_same_voice_check_and_join(
    ctx: &Context,
    msg: &Message,
    args: &mut Args,
    _command_options: &CommandOptions,
) -> Result<(), Reason> {
    println!("Join Voice decorator");
    let join_and_clear_call = does_bot_need_to_join(ctx, msg).await;

    match join_and_clear_call {
        Ok(join_and_clear) => {
            if join_and_clear {
                println!("Joining and clearning");
                join_voice(ctx, msg).await?;
                clear_messages(ctx, msg, args.clone()).await;
            }
        }
        Err(e) => {
            eprintln!("Error attempting to check if bot needs to join: {:?}", e);
        }
    }

    Ok(())
}

#[check]
#[name = "Clear_Messages"]
#[display_in_help]
// Clears message in the channel
pub async fn clear_messages_on_join(
    ctx: &Context,
    msg: &Message,
    args: &mut Args,
    _command_options: &CommandOptions,
) -> Result<(), Reason> {
    println!("Clear Messages decorator");
    clear_messages(ctx, msg, args.clone()).await;

    Ok(())
}

#[check]
#[name = "Is_Channel_DWin_Audio"]
#[display_in_help]
// Ensures command came from approved channel
pub async fn is_channel_dwin_audio(
    ctx: &Context,
    msg: &Message,
    args: &mut Args,
    _command_options: &CommandOptions,
) -> Result<(), Reason> {
    let span = span!(Level::INFO, "in_same_voice_check", ?msg);
    async move { is_channel_dwin_audio_helper(ctx, msg).await }
        .instrument(span)
        .await
}
