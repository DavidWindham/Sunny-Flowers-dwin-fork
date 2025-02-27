use serenity::{client::Context, model::id::GuildId};
use songbird::input::Restartable;
use std::env;
use tracing::instrument;

use crate::utils::{SunnyError, SunnyResult};

#[derive(Debug)]
pub enum EnqueueAt {
    Front,
    Back,
}

#[instrument(skip(ctx))]
pub async fn play(
    ctx: &Context,
    guild_id: GuildId,
    url: String,
    enqueu_at: EnqueueAt,
) -> SunnyResult<usize> {
    let spotify_client_id =
        env::var("SPOTIFY_CLIENT_ID").expect("Environment variable SPOTIFY_CLIENT_ID not found");
    println!("Spotify client ID: {}", spotify_client_id);

    let source = Restartable::ytdl(url, true).await.map_err(|e| {
        SunnyError::user_and_log(
            "Error starting stream",
            format!("Error sourcing ffmpeg {:?}", e).as_str(),
        )
    })?;

    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?;

    let call_m = songbird
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("No Call"))?;

    let mut call = call_m.lock().await;

    match enqueu_at {
        EnqueueAt::Front => {
            call.enqueue_source(source.into());
            call.queue().modify_queue(|q| {
                if let Some(track) = q.pop_back() {
                    q.push_front(track);
                    if q.len() > 1 {
                        q.swap(0, 1);
                    }
                }
            });
        }
        EnqueueAt::Back => call.enqueue_source(source.into()),
    };
    Ok(call.queue().len())
}
