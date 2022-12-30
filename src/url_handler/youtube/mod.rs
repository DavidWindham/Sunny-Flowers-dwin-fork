use reqwest::StatusCode;
use serde_json::Value;

pub async fn get_youtube_urls_from_string_vector(
    api_key: &str,
    songs: Vec<String>,
) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();

    let mut urls: Vec<String> = vec![];

    for song in songs {
        let url_call = get_url_from_song_name(&client, api_key, song.as_str()).await;
        match url_call {
            Ok(url) => {
                println!("Url for {}: {}", song, url);
                urls.push(url);
            }
            Err(e) => {
                eprintln!("Error getting url for {} - {}", song, e);
            }
        }
    }

    Ok(urls)
}

async fn get_url_from_song_name(
    client: &reqwest::Client,
    api_key: &str,
    search_term: &str,
) -> Result<String, String> {
    let mut params = [("key", api_key), ("q", &search_term)];

    let response_call = client
        .get("https://www.googleapis.com/youtube/v3/search")
        .query(&mut params)
        .send()
        .await;

    match response_call {
        Ok(response) => {
            let status_code = response.status();

            match status_code {
                StatusCode::OK => {
                    let json_call: Result<Value, reqwest::Error> = response.json().await;
                    match json_call {
                        Ok(json) => {
                            let video_id_call: &Option<&str> =
                                &json["items"][0]["id"]["videoId"].as_str();
                            match video_id_call {
                                Some(video_id) => {
                                    let video_url =
                                        format!("https://www.youtube.com/watch?v={}", video_id);
                                    Ok(video_url)
                                }
                                None => {
                                    eprintln!(
                                        "Error getting video ID as str: {}",
                                        &json["items"][0]["id"]["videoId"]
                                    );
                                    Err(format!("Error getting video ID"))
                                }
                            }
                        }
                        Err(e) => Err(format!("Error: {}", e)),
                    }
                }
                _ => Err("Error getting response".to_string()),
            }
        }
        Err(e) => {
            eprintln!("Error with response call");
            Err(format!("Error getting URL: {}", e))
        }
    }
}

pub async fn get_youtube_urls_from_playlist_url(
    api_key: &str,
    playlist_url: &str,
) -> Result<Vec<String>, String> {
    // Extract the playlist ID from the playlist URL
    let playlist_id_call = extract_playlist_id_from_url(playlist_url);
    let playlist_id;
    match playlist_id_call {
        Ok(playlist_id_unwrapped) => {
            playlist_id = playlist_id_unwrapped;
        }
        Err(e) => return Err(format!("Error getting playlist: {}", e)),
    }

    let client = reqwest::Client::new();

    let mut track_urls: Vec<String> = Vec::new();

    let mut next_page_token = "";
    let mut response_json: Value;
    loop {
        println!("In loop");
        let params = [
            ("part", "snippet"),
            ("playlistId", playlist_id),
            ("key", api_key),
            ("pageToken", next_page_token),
            ("maxResults", "50"),
        ];

        let response_body_call = client
            .get("https://www.googleapis.com/youtube/v3/playlistItems")
            .query(&params)
            .send()
            .await;

        match response_body_call {
            Ok(response_body) => {
                let response_text_call = response_body.text().await;
                match response_text_call {
                    Ok(response_text) => {
                        let response_json_call = serde_json::from_str(&response_text);
                        match response_json_call {
                            Ok(response_json_new) => response_json = response_json_new,
                            Err(e) => {
                                return Err("Error getting JSON".to_string());
                            }
                        }

                        let items_call = response_json["items"].as_array();
                        match items_call {
                            Some(items) => {
                                for item in items {
                                    let video_id_call =
                                        item["snippet"]["resourceId"]["videoId"].as_str();
                                    match video_id_call {
                                        Some(video_id) => {
                                            let track_url = format!(
                                                "https://www.youtube.com/watch?v={}",
                                                video_id
                                            );
                                            track_urls.push(track_url);
                                        }
                                        None => {
                                            eprintln!("Error parsing url");
                                        }
                                    }
                                }

                                let temp_token = response_json["nextPageToken"].as_str();

                                match temp_token {
                                    Some(s) => {
                                        next_page_token = s;
                                    }
                                    None => break,
                                };
                            }
                            None => {
                                println!("It was none");
                                return Ok(vec![]);
                            }
                        }
                        // Extract the URLs of the tracks from the JSON object
                    }
                    Err(e) => {
                        eprintln!("Error somewhere {}", e);
                    }
                }
            }
            Err(e) => {
                return Err("Error getting response body".to_string());
                // break;
            }
        }
    }

    for url in track_urls.clone() {
        println!("Url: {}", url);
    }
    Ok(track_urls)
}

fn extract_playlist_id_from_url(url: &str) -> Result<&str, &str> {
    // The playlist ID is the value of the "list" query parameter in the URL
    let query_string = url.split("?").collect::<Vec<&str>>()[1];
    let query_params = query_string.split("&").collect::<Vec<&str>>();
    for param in query_params {
        if param.starts_with("list=") {
            let playlist_id = param.split("=").collect::<Vec<&str>>()[1];
            return Ok(playlist_id);
        }
    }

    Err("Could not extract playlist ID from URL")
}
