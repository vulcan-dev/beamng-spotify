use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub height: u32,
    pub url: String,
    pub width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub album_type: String,
    pub artists: Vec<Artist>,
    pub available_markets: Vec<String>,
    pub external_urls: HashMap<String, String>,
    pub href: String,
    pub images: Vec<Image>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub total_tracks: u32,
    #[serde(rename = "type")]
    pub type_: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub uri: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artists {
    pub items: Vec<Artist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub album: Album,
    pub available_markets: Vec<String>,
    pub disc_number: u32,
    pub duration_ms: u32,
    pub explicit: bool,
    pub external_ids: HashMap<String, String>,
    pub external_urls: HashMap<String, String>,
    pub href: String,
    pub id: String,
    pub is_local: bool,
    pub name: String,
    pub popularity: u32,
    pub preview_url: Option<String>,
    pub track_number: u32,
    #[serde(rename = "type")]
    pub type_: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disallows {
    pub resuming: Option<bool>,
    pub toggling_repeat_context: Option<bool>,
    pub toggling_repeat_track: Option<bool>,
    pub toggling_shuffle: Option<bool>,
    pub skipping_prev: Option<bool>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actions {
    pub disallows: Disallows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub external_urls: HashMap<String, String>,
    pub href: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub timestamp: Option<u64>,
    pub context: Option<Context>,
    pub progress_ms: Option<u64>,
    pub item: Option<Item>,
    pub currently_playing_type: Option<String>,
    pub actions: Option<Actions>,
    pub is_playing: Option<bool>,
}