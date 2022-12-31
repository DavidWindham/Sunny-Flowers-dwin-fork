use std::num::NonZeroUsize;

use serenity::{client::Context, model::id::GuildId};
use songbird::tracks::TrackHandle;

use crate::utils::{SunnyError, SunnyResult};

use super::remove_at;

pub async fn clear(ctx: &Context, guild_id: GuildId) {
    println!("Queue clear called");
    let queue_call = get_queue(ctx, guild_id).await;
    println!("Call complete");
    match queue_call {
        Ok(queue) => {
            println!("Queue retrieved");
            let queue_length = queue.len();
            println!("Queue length: {:?}", queue_length);
            if queue_length > 0 {
                println!("Length is over 0");
                for i in (1..queue_length).rev() {
                    println!("Passed there");
                    let non_zero_index = unsafe { NonZeroUsize::new_unchecked(i) };
                    println!("Non zero processed");
                    let remove_call = remove_at(ctx, guild_id, non_zero_index).await;
                    println!("Remove call has been called");
                    match remove_call {
                        Ok(_) => {
                            println!("Song removed from queue");
                        }
                        Err(e) => {
                            eprintln!("Error removing song from queue: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error getting queue in clear: {}", e);
        }
    }
}

async fn get_queue(ctx: &Context, guild_id: GuildId) -> SunnyResult<Vec<TrackHandle>> {
    Ok(songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?
        .get(guild_id)
        .ok_or_else(|| SunnyError::user("Not currently in a call"))?
        .lock()
        .await
        .queue()
        .current_queue())
}
