use reqwest::{Client};
use serde::{Deserialize};
use std::fs::read_to_string;
use log::{warn, error};

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyReturn {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

pub async fn get_access_token() -> String {
    if !std::path::Path::new("refresh_token.txt").exists() {
        return String::new();
    }

    let client_id = dotenv::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set in .env");
    let client_secret = dotenv::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not set in .env");
    
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

    if token_response.is_empty() {
        return String::new();
    }

    let json: Result<SpotifyReturn, serde_json::Error> = serde_json::from_str(&token_response);
    if json.is_err() {
        error!("Failed getting access token, invalid json: {}\nReponse: {}", json.unwrap_err().to_string(), token_response);
        return String::new();
    }

    let json = json.unwrap();

    if let Some(access_token) = json.access_token {
        return access_token;
    }

    String::new()
}