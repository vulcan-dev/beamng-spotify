use reqwest::{Client};
use serde::{Deserialize};
use std::fs::read_to_string;

#[derive(Clone, Deserialize)]
pub struct SpotifyReturn {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

pub async fn get_access_token() -> String {
    if !std::path::Path::new("refresh_token.txt").exists() {
        return String::new();
    }

    let client_id = dotenv::var("SPOTIFY_CLIENT_ID").unwrap_or_else(|_| {
        panic!("SPOTIFY_CLIENT_ID must be set in .env file")
    });

    let client_secret = dotenv::var("SPOTIFY_CLIENT_SECRET").unwrap_or_else(|_| {
        panic!("SPOTIFY_CLIENT_SECRET must be set in .env file")
    });
    
    let buff = String::from(format!("{}:{}", client_id, client_secret));
    let base64_buff = base64::encode(&buff);
    
    let refresh_token = read_to_string("refresh_token.txt").unwrap();
    
    let token_request = Client::new()
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", format!("Basic {}", base64_buff))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
        ]);
    
    let token_response = token_request.send().await.unwrap_or_else(|e| {
        panic!("Failed sending token request: {}", e)
    }).text().await.unwrap_or_else(|e| {
        panic!("Failed getting token response: {}", e)
    });

    let json: Result<SpotifyReturn, serde_json::Error> = serde_json::from_str(&token_response);
    let json = json.unwrap();

    if let Some(access_token) = json.access_token {
        return access_token;
    }

    "".to_string()
}