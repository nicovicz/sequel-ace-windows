//! Favorites — persistensi JSON + daftar koneksi tersimpan.
//!
//! Mac memakai `Favorites.plist` (XML) di Application Support; port ini memakai
//! JSON di direktori yang sama supaya mudah di-port antar platform. Kontrak
//! field mengikuti doc `03-koneksi-favorites-keychain.md`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Favorite {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default = "default_type")]
    pub connection_type: String, // "tcpip" | "socket" | "ssh" | "aws_iam" | "vault"
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub socket_path: String,
    #[serde(default)]
    pub color_index: i32,
}

fn default_type() -> String {
    "tcpip".to_owned()
}

fn default_port() -> u16 {
    3306
}

impl Favorite {
    pub fn new(
        name: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        user: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            connection_type: "tcpip".to_owned(),
            host: host.into(),
            port,
            user: user.into(),
            password: String::new(),
            database: String::new(),
            socket_path: String::new(),
            color_index: -1,
        }
    }
}

/// Loader/saver JSON di Application Support (atau %APPDATA% di Windows).
pub struct FavoritesStore {
    path: PathBuf,
}

impl FavoritesStore {
    pub fn new(app_support_dir: PathBuf) -> Self {
        let dir = app_support_dir.join("Sequel Ace");
        let _ = std::fs::create_dir_all(&dir);
        Self {
            path: dir.join("favorites.json"),
        }
    }

    pub fn load(&self) -> Vec<Favorite> {
        let Ok(text) = std::fs::read_to_string(&self.path) else {
            return Vec::new();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self, favorites: &[Favorite]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(favorites)
            .map_err(|error| format!("Could not serialize favorites: {error}"))?;
        std::fs::write(&self.path, json)
            .map_err(|error| format!("Could not write favorites: {error}"))
    }
}
