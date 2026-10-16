//! Query favorites + query history — JSON persistence.
//!
//! Mac memakai SQLite (`SQLiteHistoryManager`) untuk history dan
//! `NSUserDefaults` key `queryFavorites` untuk favorites. Port ini memakai
//! JSON di Application Support agar konsisten dengan connection favorites.
//! Kontrak: doc `08b-query-history-favorites.md`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Satu query favorite (mac: `SPQueryFavoriteManager.favorites` array flat).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryFavorite {
    pub id: String,
    pub name: String,
    pub sql: String,
}

/// Query history entry (mac: `QueryHistory` table, doc 08b §1.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistoryEntry {
    pub query: String,
    pub created_time: f64,
}

/// Store untuk query favorites + history.
pub struct QueryStore {
    favorites_path: PathBuf,
    history_path: PathBuf,
    /// Max entri history (mac default 20, doc 08b §1.3).
    pub max_history: usize,
}

impl QueryStore {
    pub fn new(app_support_dir: PathBuf) -> Self {
        let dir = app_support_dir.join("Sequel Ace").join("Data");
        let _ = std::fs::create_dir_all(&dir);
        Self {
            favorites_path: dir.join("query_favorites.json"),
            history_path: dir.join("query_history.json"),
            max_history: 20,
        }
    }

    // --- Favorites ---

    pub fn load_favorites(&self) -> Vec<QueryFavorite> {
        let Ok(text) = std::fs::read_to_string(&self.favorites_path) else {
            return Vec::new();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save_favorites(&self, favorites: &[QueryFavorite]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(favorites)
            .map_err(|error| format!("Could not serialize query favorites: {error}"))?;
        std::fs::write(&self.favorites_path, json)
            .map_err(|error| format!("Could not write query favorites: {error}"))
    }

    // --- History ---

    pub fn load_history(&self) -> Vec<QueryHistoryEntry> {
        let Ok(text) = std::fs::read_to_string(&self.history_path) else {
            return Vec::new();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    /// Tambah query ke history (dedup + cap, doc 08b §1.5).
    pub fn add_history(&self, query: &str) {
        let mut history = self.load_history();
        // Dedup: duplikat dibuang (mac `appendIfNotContains`).
        history.retain(|entry| entry.query != query);
        // Insert di depan (mac: `insertObject:atIndex:0`).
        history.insert(
            0,
            QueryHistoryEntry {
                query: query.to_owned(),
                created_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64(),
            },
        );
        // Cap (mac default 20).
        history.truncate(self.max_history);
        if let Ok(json) = serde_json::to_string_pretty(&history) {
            let _ = std::fs::write(&self.history_path, json);
        }
    }

    pub fn clear_history(&self) {
        let _ = std::fs::write(&self.history_path, "[]");
    }
}
