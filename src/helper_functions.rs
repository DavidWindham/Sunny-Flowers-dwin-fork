use serenity::{
    framework::standard::{Args, Reason},
    futures::stream::{self, StreamExt},
    model::prelude::Message,
    prelude::Context,
};

use crate::{effects, structs::EventConfig, utils::SunnyError};

pub async fn does_bot_need_to_join(ctx: &Context, msg: &Message) -> Result<bool, SunnyError> {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    // The user's voice channel id
    let voice_channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|vs| vs.channel_id)
        .ok_or_else(|| SunnyError::user("Not in a voice"))?;

    let bot_id = ctx.cache.current_user_id().await;
    let same_voice = guild
        .voice_states
        .get(&bot_id)
        .and_then(|vs| vs.channel_id)
        .map_or(false, |id| id == voice_channel_id);

    if same_voice {
        return Ok(false);
    }

    return Ok(true);
}

pub async fn join_voice(ctx: &Context, msg: &Message) -> Result<(), SunnyError> {
    println!("Join Voice helper function called");
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    // The user's voice channel id
    let voice_channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|vs| vs.channel_id)
        .ok_or_else(|| SunnyError::user("Not in a voice"))?;

    let call_m = effects::join(&EventConfig {
        ctx: ctx.clone(),
        guild_id: guild.id,
        text_channel_id: msg.channel_id,
        voice_channel_id,
    })
    .await?;

    effects::deafen(call_m).await;

    Ok(())
}

pub async fn clear_messages(
    ctx: &Context,
    msg: &Message,
    _args: Args,
) -> Result<(), serenity::Error> {
    println!("Clear Message helper function called");
    let channel_id = msg.channel_id;

    let messages = channel_id
        .messages(&ctx.http, |retriever| retriever.limit(25))
        .await?;

    let message_stream = stream::iter(messages).fuse();

    let message_ids = message_stream
        .fold(Vec::new(), |mut acc, single_message| async move {
            if !single_message.is_own(&ctx.cache).await {
                acc.push(single_message.id);
            } else {
                acc.extend(vec![single_message.id]);
            }
            acc
        })
        .await;

    println!("Messages: {:?}", message_ids);

    channel_id.delete_messages(&ctx.http, message_ids).await?;

    Ok(())
}

// async fn filter_message(
//     ctx: &Context,
//     single_message: &Message,
//     bot_message_id: MessageId,
// ) -> bool {
//     if single_message.id != bot_message_id {
//         if try_join!(check_message_is_own(ctx, single_message).await) {
//             return true;
//         }
//     }

//     return false;
// }

// async fn check_message_is_own(ctx: &Context, msg: &Message) -> bool {
//     let is_own_call = msg.is_own(&ctx.cache).await;
//     return is_own_call;
// }

pub async fn is_channel_dwin_audio_helper(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    let _songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Failed to get songbird"))?;

    let channel = {
        msg.channel(&ctx.cache)
            .await
            .ok_or_else(|| SunnyError::log("Unable to fetch channel"))?
    };

    let channel_guild = {
        channel
            .clone()
            .guild()
            .ok_or_else(|| SunnyError::log("Unable to fetch guild"))?
    };

    if channel_guild.name == "dwin_audio" {
        return Ok(());
    }

    return Err(Reason::User(
        "Cannot take commands in this channel. Please create a text channel named 'dwin_audio'"
            .to_string(),
    ));
}
