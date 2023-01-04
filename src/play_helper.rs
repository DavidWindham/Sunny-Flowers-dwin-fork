use crate::url_handler::spotify;
use crate::url_handler::youtube;

use std::env;

pub async fn get_urls(url: String) -> Result<Vec<String>, String> {
    if url.starts_with("https://youtu.be/") || url.starts_with("https://www.youtube.com/watch") {
        return Ok(vec![url]);
    }

    if url.starts_with("https://open.spotify.com/playlist") {
        println!("This is a Spotify playlist");
        let spotify_client_id = env::var("SPOTIFY_CLIENT_ID")
            .expect("Environment variable SPOTIFY_CLIENT_ID not found");
        let spotify_client_secret = env::var("SPOTIFY_CLIENT_SECRET")
            .expect("Environment variable SPOTIFY_SECRET not found");
        let youtube_token =
            env::var("YOUTUBE_TOKEN").expect("Environment variable YOUTUBE_TOKEN not found");
        return get_urls_for_spotify_playlist(
            spotify_client_id,
            spotify_client_secret,
            youtube_token,
            url,
        )
        .await;
    }

    if url.starts_with("https://www.youtube.com/playlist") {
        println!("This is a YouTube playlist");
        let youtube_key =
            env::var("YOUTUBE_TOKEN").expect("Environment variable YOUTUBE_TOKEN not found");
        return get_urls_for_youtube_playlist(youtube_key, url).await;
    }

    Err("No URL's found".to_string())
}

pub async fn get_urls_for_search_term(search_term: String) -> Result<Vec<String>, String> {
    let youtube_key =
        env::var("YOUTUBE_TOKEN").expect("Environment variable YOUTUBE_TOKEN not found");
    let url_call = get_url_for_search_term(youtube_key, search_term).await;
    match url_call {
        Ok(urls) => Ok(urls),
        Err(e) => Err(format!("No URL's found: {}", e)),
    }
}

async fn get_url_for_search_term(
    youtube_key: String,
    search_term: String,
) -> Result<Vec<String>, String> {
    let urls_call =
        youtube::get_youtube_urls_from_string_vector(youtube_key.as_str(), vec![search_term]).await;

    match urls_call {
        Ok(urls) => Ok(urls),
        Err(e) => {
            eprintln!("Error getting URLs for search term: {}", e);
            Err(e)
        }
    }
}

async fn get_urls_for_spotify_playlist(
    client_id: String,
    client_secret: String,
    youtube_key: String,
    url: String,
) -> Result<Vec<String>, String> {
    let songs = spotify::get_spotify_playlist_tracks(
        client_id.as_str(),
        client_secret.as_str(),
        url.as_str(),
    )
    .await;

    let urls_call = youtube::get_youtube_urls_from_string_vector(youtube_key.as_str(), songs).await;

    match urls_call {
        Ok(urls) => Ok(urls),
        Err(e) => {
            eprintln!("Error getting URLs: {}", e);
            Err(e)
        }
    }
}

async fn get_urls_for_youtube_playlist(
    youtube_key: String,
    url: String,
) -> Result<Vec<String>, String> {
    let url_call =
        youtube::get_youtube_urls_from_playlist_url(youtube_key.as_str(), url.as_str()).await;
    match url_call {
        Ok(urls) => Ok(urls),
        Err(e) => {
            eprintln!("Error getting youtube urls from playlist url: {}", e);
            Err(e)
        }
    }
}
