use std::{collections::HashSet, num::NonZeroUsize};

use serenity::{
    client::Context,
    framework::standard::{
        help_commands,
        macros::{command, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::prelude::*,
};

use url::Url;

use crate::{
    checks::*,
    effects::{
        self, display_queue, now_playing,
        queue::{self, EnqueueAt},
    },
    structs::EventConfig,
    utils::SunnyError,
};

#[help]
pub async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(ctx, msg, args, help_options, groups, owners)
        .await
        .ok_or_else(|| SunnyError::log("failed to send"))?;
    Ok(())
}

/*
#[command]
#[only_in(guilds)]
/// Adds Sunny to the user's current voice channel.
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
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
        return Err(SunnyError::user("Already in that voice channel!").into());
    }

    let call_m = effects::join(&EventConfig {
        ctx: ctx.clone(),
        guild_id: guild.id,
        text_channel_id: msg.channel_id,
        voice_channel_id,
    })
    .await?;

    effects::deafen(call_m).await;
    msg.channel_id
        .say(&ctx.http, format!("Joined {}", voice_channel_id.mention()))
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Removes Sunny from the current voice channel and clears the queue.
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get guild"))?;

    effects::leave(ctx, guild.id).await?;

    msg.reply(&ctx.http, "Left voice").await?;

    Ok(())
}
*/

fn validate_url(mut args: Args) -> Option<String> {
    let mut url: String = args.single().ok()?;

    if url.starts_with('<') && url.ends_with('>') {
        url = url[1..url.len() - 1].to_string();
    }

    Url::parse(&url).ok()?;

    Some(url)
}

#[command]
#[aliases(p)]
#[max_args(1)]
#[only_in(guilds)]
#[usage("<url>")]
#[example("https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
#[checks(Is_Channel_DWin_Audio)]
// #[checks(In_Voice)]
// #[checks(Clear_Messages)]
#[checks(Join_Voice)]
/// While Sunny is in a voice channel, you may run the play command so that she
/// can start streaming the given video URL.
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let url = validate_url(args).ok_or_else(|| SunnyError::user("Unable to parse url"))?;

    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    let len = queue::play(ctx, guild_id, url, EnqueueAt::Back).await?;

    let reply = if len == 1 {
        "Started playing the song".to_string()
    } else {
        format!("Added song to queue: position {}", len - 1)
    };

    // msg.reply(&ctx.http, reply).await?;

    Ok(())
}

/*
#[command]
#[aliases(pn)]
#[max_args(1)]
#[only_in(guilds)]
#[usage("<url>")]
#[example("https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
#[checks(In_Voice)]
/// While Sunny is in a voice channel, you may run the play command so that she
/// can start streaming the given video URL.
pub async fn play_next(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let url = validate_url(args).ok_or_else(|| SunnyError::user("Unable to parse url"))?;

    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    queue::play(ctx, guild_id, url, EnqueueAt::Front).await?;

    msg.reply(&ctx.http, "Added song to front of queue").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
/// Shuffles your queue badly
pub async fn shuffle(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("Failed to get guild id"))?;

    queue::shuffle(ctx, guild_id).await?;
    msg.reply(&ctx.http, "Queue Shuffled :game_die:!").await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[min_args(2)]
#[max_args(2)]
#[usage("<position> <position>")]
#[example("4 2")]
/// Swaps two songs in the queue by their number
pub async fn swap(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("Failed to get guild id"))?;

    let a = args
        .single::<NonZeroUsize>()
        .map_err(|_| SunnyError::user("Invalid arguments"))?;

    let b = args
        .single::<NonZeroUsize>()
        .map_err(|_| SunnyError::user("Invalid arguments"))?;

    let (t1, t2) = queue::swap(ctx, guild_id, a.into(), b.into()).await?;

    msg.reply(
        &ctx.http,
        format!(
            "Swapped `{}` and `{}`",
            effects::get_song(t1.metadata()),
            effects::get_song(t2.metadata())
        ),
    )
    .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(np)]
/// Shows the currently playing media
pub async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    now_playing::send_embed(ctx, guild_id, msg.channel_id).await?;

    msg.delete(&ctx.http).await?;

    Ok(())
}
 */

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Pauses the currently playing
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    queue::pause(ctx, guild_id).await?;

    // msg.reply(&ctx.http, "Track paused").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Resumes the current song if it was paused
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    queue::resume(ctx, guild_id).await?;

    // msg.reply(&ctx.http, "Track resumed").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Skips the currently playing song and starts the next song in the queue.
pub async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    let len = queue::skip(ctx, guild_id).await?;

    // msg.reply(
    //     &ctx.http,
    //     format!(
    //         "Song skipped: {} in queue.",
    //         len.checked_sub(1).unwrap_or_default()
    //     ),
    // )
    // .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Stops playing the current song and clears the current song queue.
pub async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    queue::stop(ctx, guild_id).await?;

    // msg.reply(&ctx.http, "Queue cleared.").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(q, queueueueu)]
/// Shows the current queue
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Queue called");
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    display_queue::send_embed(ctx, guild_id, msg.channel_id).await?;
    println!("Queue end?");
    Ok(())
}

/*
#[command]
#[only_in(guilds)]
#[aliases(r, remove)]
#[max_args(1)]
#[example("2")]
#[usage("<position>")]
/// Removes a song from the queue by its position
pub async fn remove_at(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    #[allow(clippy::unwrap_used)]
    let index = args
        .single::<NonZeroUsize>()
        .unwrap_or_else(|_| NonZeroUsize::new(1).unwrap());

    let q = queue::remove_at(ctx, guild_id, index).await?;

    msg.reply(
        &ctx.http,
        format!("Removed: `{}`", effects::get_song(q.metadata())),
    )
    .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
/// Pong
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;
    Ok(())
}
*/
