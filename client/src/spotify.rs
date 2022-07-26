use serde::{Serialize, Deserialize};
use reqwest::Client;
use actix_web::{get, post, Responder, HttpResponse, web};
use std::fs::read_to_string;

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyOffset {
    pub position: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyPlay {
    pub context_uri: Option<String>,
    pub offset: SpotifyOffset,
    pub position_ms: u32
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
async fn spotify_play(body: web::Json<SpotifyPlay>) -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    client
        .put("https://api.spotify.com/v1/me/player/play")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Length", "0")
        .body(serde_json::to_string(&body).unwrap())
        .send().await.unwrap().text().await.unwrap();

    HttpResponse::Ok().finish()
}

#[post("/api/v1/pause_song")]
async fn spotify_pause() -> impl Responder {
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
async fn spotify_seek(position_ms: web::Path<u32>) -> impl Responder {
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

    HttpResponse::Ok().finish()
}

#[post("/api/v1/volume/{volume}")]
async fn spotify_volume(volume: web::Path<u32>) -> impl Responder {
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

    HttpResponse::Ok().finish()
}

#[get("/api/v1/active_device")]
async fn active_device() -> impl Responder {
    let access_token = client::get_access_token().await;
    if access_token.is_empty() {
        return HttpResponse::Ok().body("No access token");
    }

    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    let response = client
        .get("https://api.spotify.com/v1/me/player")
        .header("Authorization", format!("Bearer {}", access_token))
        .send().await.unwrap().text().await.unwrap();

    HttpResponse::Ok().body(response)
}