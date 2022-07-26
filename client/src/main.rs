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
mod spotify;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

#[get("/login")]
async fn login() -> impl Responder {
    let scope = String::from("user-read-currently-playing user-modify-playback-state playlist-read-private playlist-read-collaborative user-read-playback-state");
    let redirect_uri = String::from("http://localhost:8888/api/v1/callback");
    let client_id = dotenv::var("SPOTIFY_CLIENT_ID").unwrap_or_else(|_| {
        panic!("SPOTIFY_CLIENT_ID must be set in .env file")
    });

    let redirect_url = format!("https://accounts.spotify.com/authorize?response_type=code&client_id={}&scope={}&redirect_uri={}", client_id, scope, redirect_uri);
    HttpResponse::Found().append_header(("Location", redirect_url)).finish()
}

#[get("/api/v1/callback")]
async fn callback(info: web::Query<AuthRequest>) -> impl Responder {
    let code = info.code.clone();

    let client_id = dotenv::var("SPOTIFY_CLIENT_ID").unwrap_or_else(|_| {
        panic!("SPOTIFY_CLIENT_ID must be set in .env file")
    });

    let client_secret = dotenv::var("SPOTIFY_CLIENT_SECRET").unwrap_or_else(|_| {
        panic!("SPOTIFY_CLIENT_SECRET must be set in .env file")
    });
    
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

    // check if the values are empty
    let client_id = dotenv::var("SPOTIFY_CLIENT_ID").unwrap_or_else(|_| {
        panic!("SPOTIFY_CLIENT_ID must be set in .env file")
    });

    let client_secret = dotenv::var("SPOTIFY_CLIENT_SECRET").unwrap_or_else(|_| {
        panic!("SPOTIFY_CLIENT_SECRET must be set in .env file")
    });

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
        loop {
            let access_token = client::get_access_token().await;
            if access_token.is_empty() {
                return;
            }
        
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
                error!("Error parsing JSON: {}\n{}", result.unwrap_err(), token_response);
                return;
            } else {
                let json: song::Song = serde_json::from_str(&token_response).unwrap();
            
                use std::path::Path;
            
                if !Path::new("song.json").exists() {
                    let mut file = File::create("song.json").unwrap_or_else(|e| {
                        panic!("Error opening song.json: {}", e.to_string())
                    });

                    file.write_all(serde_json::to_string(&json).unwrap().as_bytes()).unwrap_or_else(|e| {
                        panic!("Error writing to song.json: {}", e.to_string())
                    });
                } else {
                    let file_str = read_to_string("song.json").unwrap_or_else(|e| {
                        panic!("Error reading song.json: {}", e.to_string())
                    });

                    let file_json: song::Song = serde_json::from_str(&file_str).unwrap_or_else(|e| {
                        panic!("Error parsing song.json: {}", e.to_string())
                    });
                    
                    if let (Some(item1), Some(item2)) = (json.clone().item, file_json.clone().item) {
                        if item1.name != item2.name {
                            info!("Song changed to {}", item1.name);
                        }
                    }

                    if json.progress_ms != file_json.progress_ms || json.is_playing != file_json.is_playing {
                        let mut file = File::create("song.json").unwrap_or_else(|e| {
                            panic!("Error opening song.json: {}", e.to_string())
                        });

                        file.write_all(serde_json::to_string(&json).unwrap_or_else(|e| {
                            panic!("Error writing to song.json: {}", e.to_string())
                        }).as_bytes()).unwrap_or_else(|e| {
                            panic!("Error writing to song.json: {}", e.to_string())
                        });   
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });

    let task_server = HttpServer::new(|| {
        App::new()
            .service(callback)
            .service(login)
            .service(spotify::current_song)
            .service(spotify::next_song)
            .service(spotify::previous_song)
            .service(spotify::spotify_play)
            .service(spotify::spotify_pause)
            .service(spotify::spotify_seek)
            .service(spotify::spotify_volume)
            .service(spotify::active_device)
    });

    use std::net::SocketAddr;
    let addr = SocketAddr::from(([127, 0, 0, 1], 8888));
    task_server.bind(addr).unwrap_or_else(|e| {
        panic!("Error binding to {}: {}", addr, e);
    }).run().await.unwrap_or_else(|e| {
        panic!("Error running server: {}", e);
    });
}