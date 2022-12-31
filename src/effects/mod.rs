//! # Effects
//! Effects contains the main functionality of Sunny
//!

mod deafen;
pub mod display_queue;
mod join;
mod leave;
pub mod now_playing;
pub mod queue;

pub use deafen::deafen;
pub use join::join;
pub use leave::leave;

use songbird::input::Metadata;
use std::time::Duration;

/// `split_duration` splits a [`Duration`] into a (minutes, seconds) tuple
const fn split_duration(d: Duration) -> (u64, u64) {
    (d.as_secs() / 60, d.as_secs() % 60)
}

fn get_title(m: &Metadata) -> &str {
    m.track
        .as_deref()
        .or_else(|| m.title.as_deref())
        .unwrap_or("Unknown Title")
}

fn get_artist(m: &Metadata) -> &str {
    m.artist
        .as_deref()
        .or_else(|| m.channel.as_deref())
        .unwrap_or("Unknown Artist")
}

pub fn _get_song(m: &Metadata) -> String {
    format!("{} by {}", get_title(m), get_artist(m))
}

const fn string_or_default<'a>(s: &'a str, d: &'a str) -> &'a str {
    if s.is_empty() {
        d
    } else {
        s
    }
}
