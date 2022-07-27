use log::info;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use actix_web::{get, post, Responder, HttpResponse, web};
use std::fs::read_to_string;

use crate::device;

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyOffset {
    pub position: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyPlay {
    pub uris: Option<Vec<String>>,
    pub context_uri: Option<String>,
    pub offset: Option<SpotifyOffset>,
    pub position_ms: Option<u32>
}

#[get("/api/v1/current_song")]
async fn current_song() -> impl Responder {
    let current_song = read_to_string("song.json").unwrap();
    HttpResponse::Ok().body(current_song)
}

#[post("/api/v1/next_song")]
async fn next_song() -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    client
        .post("https://api.spotify.com/v1/me/player/next")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Content-Length", "0")
        .send().await.unwrap().text().await.unwrap();

    HttpResponse::Ok().finish()
}

#[post("/api/v1/previous_song")]
async fn previous_song() -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    client
        .post("https://api.spotify.com/v1/me/player/previous")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Length", "0")
        .send().await.unwrap().text().await.unwrap();

    let yes = read_to_string("song.json").unwrap();
    HttpResponse::Ok().body(yes)
}

#[post("/api/v1/play_song")]
async fn play(body: web::Json<SpotifyPlay>) -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    let json = serde_json::to_string(&body).unwrap();

    client
        .put("https://api.spotify.com/v1/me/player/play")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Content-Length", format!("{}", json.len()))
        .body(json)
        .send().await.unwrap().text().await.unwrap();

    HttpResponse::Ok().finish()
}

#[post("/api/v1/pause_song")]
async fn pause() -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();
        

    client
        .put("https://api.spotify.com/v1/me/player/pause")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Length", "0")
        .send().await.unwrap().text().await.unwrap();

    HttpResponse::Ok().finish()
}

#[post("/api/v1/seek/{position_ms}")]
async fn seek(position_ms: web::Path<u32>) -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let position_ms = position_ms;

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    client
        .put(format!("https://api.spotify.com/v1/me/player/seek/?position_ms={}", position_ms))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Length", "0")
        .send().await.unwrap().text().await.unwrap();

    let position_ms_i64 = position_ms.into_inner() as i64;

    let mins: i64 = position_ms_i64 / 1000 / 60;
    let secs: i64 = (position_ms_i64 / 1000) % 60;

    let min_str = if mins < 10 {
        format!("0{}", mins)
    } else {
        format!("{}", mins)
    };

    let sec_str = if secs < 10 {
        format!("0{}", secs)
    } else {
        format!("{}", secs)
    };

    let time = format!("{}:{}", min_str, sec_str);

    info!("Set time to: {}", time);

    HttpResponse::Ok().finish()
}

#[post("/api/v1/volume/{volume}")]
async fn volume(volume: web::Path<u32>) -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let volume = volume;

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    client
        .put(format!("https://api.spotify.com/v1/me/player/volume/?volume_percent={}", volume))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Length", "0")
        .send().await.unwrap().text().await.unwrap();

    info!("Set volume to: {}", volume);

    HttpResponse::Ok().finish()
}

#[get("/api/v1/playlists")]
async fn playlists() -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    let response = client
        .get("https://api.spotify.com/v1/me/playlists")
        .header("Authorization", format!("Bearer {}", access_token))
        .send().await.unwrap();

    HttpResponse::Ok().body(response.text().await.unwrap())
}
// todo: make a new thread for playlist and tracks, update every 30 seconds.
#[get("/api/v1/playlists/{playlist_id}/tracks")]
async fn playlist_tracks(playlist_id: web::Path<String>) -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    let response = client
        .get(format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist_id))
        .header("Authorization", format!("Bearer {}", access_token))
        .send().await.unwrap();

    HttpResponse::Ok().body(response.text().await.unwrap())
}

#[get("/api/v1/albums")]
async fn albums() -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    let response = client
        .get("https://api.spotify.com/v1/me/albums")
        .header("Authorization", format!("Bearer {}", access_token))
        .send().await.unwrap();

    HttpResponse::Ok().body(response.text().await.unwrap())
}

#[get("/api/v1/top_tracks")]
async fn top_tracks() -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    let response = client
        .get("https://api.spotify.com/v1/me/top/tracks")
        .header("Authorization", format!("Bearer {}", access_token))
        .send().await.unwrap();

    HttpResponse::Ok().body(response.text().await.unwrap())
}

#[get("/api/v1/active_device")]
async fn active_device() -> impl Responder {
    if let Some(active_device) = read_to_string("active_device.json").ok() {
        return HttpResponse::Ok().body(active_device);
    }

    HttpResponse::Ok().body(serde_json::to_string(&device::SpotifyDevice::default()).unwrap())
}