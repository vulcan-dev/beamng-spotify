use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub is_active: bool,
    pub is_private_session: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub volume_percent: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyDevice {
    pub device: Option<Device>,
}

impl Default for SpotifyDevice {
    fn default() -> Self {
        SpotifyDevice {
            device: None,
        }
    }
}