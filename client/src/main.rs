extern crate pretty_env_logger;

use std::{fs::File, io::Read};
use std::io::Write;

use actix_web::{get, Responder, HttpResponse, web, HttpServer, App};
use serde::Deserialize;
use reqwest::Client;
use std::fs::read_to_string;
use pretty_env_logger::env_logger;
use std::path::Path;

use log::{info, error};

mod song;
mod device;
mod spotify;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

#[get("/login")]
async fn login() -> impl Responder {
    let scope = String::from("user-read-currently-playing user-modify-playback-state playlist-read-private playlist-read-collaborative user-read-playback-state user-library-read user-modify-playback-state user-top-read");
    let redirect_uri = String::from("http://localhost:8888/api/v1/callback");
    let client_id = dotenv::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set in .env");

    let redirect_url = format!("https://accounts.spotify.com/authorize?response_type=code&client_id={}&scope={}&redirect_uri={}", client_id, scope, redirect_uri);
    HttpResponse::Found().append_header(("Location", redirect_url)).finish()
}

#[get("/api/v1/callback")]
async fn callback(info: web::Query<AuthRequest>) -> impl Responder {
    let code = info.code.clone();

    let client_id = dotenv::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set in .env");
    let client_secret = dotenv::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not set in .env");
    
    let buff = String::from(format!("{}:{}", client_id, client_secret));
    let base64_buff = base64::encode(&buff);

    let token_request = Client::new()
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", format!("Basic {}", base64_buff))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", "http://localhost:8888/api/v1/callback"),
        ]);

    let token_response = token_request.send().await.unwrap().text().await.unwrap();
    let json: client::SpotifyReturn = serde_json::from_str(&token_response).unwrap();

    let refresh_token = json.refresh_token.clone();

    use std::fs::File;

    if !Path::new("refresh_token.txt").exists() {
        File::create("refresh_token.txt").unwrap();
    }

    if refresh_token.is_some() {
        let mut file = File::create("refresh_token.txt").unwrap();
        file.write_all(refresh_token.unwrap().as_bytes()).unwrap();
    }

    info!("Got refresh token, you can now close the browser window and continue...");

    HttpResponse::Ok().body("Got refresh token, you can now close the browser window and continue...")
}

async fn write_active_song(access_token: &str) {
    let token_request = Client::new()
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/x-www-form-urlencoded");

    let token_response = token_request
        .send().await.unwrap_or_else(|e| {
            panic!("Failed sending request to \"https://api.spotify.com/v1/me/player/currently-playing\": {}", e)
        }).text().await.unwrap_or_else(|e| {
            panic!("Failed getting text response from \"https://api.spotify.com/v1/me/player/currently-playing\": {}", e)
        });

    if token_response.is_empty() {
        return;
    }

    let result: Result<song::Song, serde_json::Error> = serde_json::from_str(&token_response);
    if result.is_err() {
        error!("(write_active_song) Error parsing JSON: {}\n{}", result.unwrap_err(), token_response);
    } else {
        let json: song::Song = serde_json::from_str(&token_response).unwrap();

        if !Path::new("song.json").exists() {
            let mut file = File::create("song.json").unwrap_or_else(|e| {
                panic!("Error opening song.json: {}", e.to_string())
            });

            file.write_all(serde_json::to_string(&json).unwrap().as_bytes()).expect("Error writing to song.json");
        } else {
            let file_str = read_to_string("song.json").expect("Error reading song.json");
            let file_json: song::Song = serde_json::from_str(&file_str).expect("Error parsing song.json");

            if json.progress_ms != file_json.progress_ms || json.is_playing != file_json.is_playing {
                let mut file = File::create("song.json").unwrap_or_else(|e| {
                    panic!("Error opening song.json: {}", e.to_string())
                });

                file.write_all(serde_json::to_string(&json).expect("Error writing to song.json").as_bytes()).expect("Error writing to song.json");

                if let (Some(item1), Some(item2)) = (json.clone().item, file_json.clone().item) {
                    if item1.name != item2.name {
                        info!("Song changed to \"{}\"", item1.name);
                    }
                }
            }
        }
    }
}

async fn write_active_device(access_token: &str) -> bool {
    let client = Client::builder()
        .user_agent("BeamNG-Spotify")
        .build().unwrap();

    let response = client
        .get("https://api.spotify.com/v1/me/player")
        .header("Authorization", format!("Bearer {}", access_token))
        .send().await.unwrap().text().await.unwrap();

    if response.is_empty() {
        return false;
    }

    let result: Result<device::SpotifyDevice, serde_json::Error> = serde_json::from_str(&response);
    if result.is_err() {
        error!("(write_active_device) Error parsing JSON: {}\nReceived:\n{}", result.unwrap_err(), response);
    } else {
        let json: device::SpotifyDevice = serde_json::from_str(&response).unwrap();

        let mut file = File::create("active_device.json").unwrap_or_else(|e| {
            panic!("Error opening active_device.json: {}", e.to_string())
        });
    
        file.write_all(serde_json::to_string(&json).unwrap().as_bytes()).unwrap_or_else(|e| {
            panic!("Error writing to active_device.json: {}", e.to_string())
        });
    }

    return true;
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting up...");

    if !Path::new(".env").exists() {
        let mut file = File::create(".env").unwrap();
        file.write_all("SPOTIFY_CLIENT_ID=\nSPOTIFY_CLIENT_SECRET=".as_bytes()).unwrap();
        info!("Created .env file, please fill in the values
Steps:
    1. Open https://developer.spotify.com/dashboard/login
    2. Create a new app and give it a name and description, I called mine \"BeamNG-Spotify\"
    3. Copy the client ID and client secret into the .env file
        SPOTIFY_CLIENT_ID=<client_id>
        SPOTIFY_CLIENT_SECRET=<client_secret>
    4. Run the client again
");
        let mut stdin = std::io::stdin();
        let _ = stdin.read(&mut [0u8]).unwrap();
        std::process::exit(0);
    }

    let client_id = dotenv::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set in .env");
    let client_secret = dotenv::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not set in .env");

    if client_id.is_empty() || client_secret.is_empty() {
        error!("SPOTIFY_CLIENT_ID and SPOTIFY_CLIENT_SECRET must be set in .env file");
        let mut stdin = std::io::stdin();
        let _ = stdin.read(&mut [0u8]).unwrap();
        std::process::exit(0);
    }

    if !Path::new("refresh_token.txt").exists() {
        info!("Opening browser to get code...");
        let _ = open::that("http://localhost:8888/login");
    }

    let _ = tokio::spawn(async {
        let mut device_offline = true;

        let access_token = client::get_access_token().await;
        if access_token.is_empty() {
            error!("Error getting access token, please make sure your .env file is correct.");
            std::process::exit(1);
        }

        info!("Access token: {}", access_token);

        loop {        
            write_active_song(&access_token).await;
            let new_online = write_active_device(&access_token).await;
            if !device_offline && !new_online {
                device_offline = true;
                info!("Device went offline");
            } else if device_offline && new_online {
                device_offline = false;
                info!("Device is online");
            }

            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });

    HttpServer::new(|| {
        App::new()
            .service(callback)
            .service(login)
            .service(spotify::current_song)
            .service(spotify::next_song)
            .service(spotify::previous_song)
            .service(spotify::play)
            .service(spotify::pause)
            .service(spotify::seek)
            .service(spotify::volume)
            .service(spotify::active_device)
            .service(spotify::playlists)
            .service(spotify::playlist_tracks)
            .service(spotify::albums)
            .service(spotify::top_tracks)
    }).workers(2).bind("localhost:8888").unwrap_or_else(|e| {
        panic!("Failed to bind to localhost:8888: {}", e.to_string())
    }).run().await.unwrap_or_else(|e| {
        panic!("Failed to run server: {}", e.to_string())
    });
}