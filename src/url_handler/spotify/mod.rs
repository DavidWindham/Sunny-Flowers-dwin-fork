use rspotify::{clients::BaseClient, model::PlaylistId, ClientCredsSpotify, Credentials};
use rspotify_model::{FullTrack, PlayableItem};

use url::Url;

fn extract_playlist_id(url: &str) -> Option<String> {
    let parsed_url = Url::parse(url).ok()?;
    let path = parsed_url.path();
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() == 3 && segments[1] == "playlist" {
        Some(segments[2].to_string())
    } else {
        None
    }
}

pub async fn get_spotify_playlist_tracks(
    client_id: &str,
    client_secret: &str,
    url: &str,
) -> Vec<String> {
    let playlist_id_call = extract_playlist_id(url);

    let mut songs: Vec<String> = vec![];

    match playlist_id_call {
        Some(playlist_id) => {
            println!("PLaylist ID: {:?}", playlist_id);
            let tracks_call = get_playlist_tracks(client_id, client_secret, playlist_id).await;
            match tracks_call {
                Ok(tracks) => {
                    for track in tracks {
                        println!("Track: {} - {}", track.name, track.album.name);
                        let track_name = track.name;
                        let mut track_artist = "";
                        if track.artists.len() > 0 {
                            track_artist = track.artists[0].name.as_str()
                        }
                        songs.push(format!("{} - {}", track_name, track_artist));
                    }
                }
                Err(e) => {
                    eprintln!("Error getting tracks: {:?}", e);
                }
            }
        }
        None => {
            eprintln!("Playlist ID was none");
        }
    }

    songs
}

async fn get_playlist_tracks(
    client_id: &str,
    client_secret: &str,
    playlist_id: String,
) -> Result<Vec<FullTrack>, String> {
    let credentials = Credentials::new(client_id, client_secret);
    let spotify = ClientCredsSpotify::new(credentials);
    let token_request = spotify.request_token().await;

    match token_request {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("Error getting token: {}", e));
        }
    }

    let mut tracks: Vec<FullTrack> = vec![];

    let playlist_id_call = PlaylistId::from_id(playlist_id);
    match playlist_id_call {
        Ok(playlist_id) => {
            let playlist_call = spotify.playlist(playlist_id, None, None).await;

            match playlist_call {
                Ok(playlist) => {
                    for playlist_item in playlist.tracks.items {
                        let single_track_call = playlist_item.track;
                        match single_track_call {
                            Some(playable_item) => match playable_item {
                                PlayableItem::Track(track) => {
                                    tracks.push(track);
                                }
                                PlayableItem::Episode(_episode) => {}
                            },
                            None => {}
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Error Getting Playlist: {}", e));
                }
            }
        }
        Err(e) => {
            eprintln!("Error getting PlaylistId from id");
        }
    }

    Ok(tracks)
}
