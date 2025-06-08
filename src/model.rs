use serde::{Serialize, Deserialize};
use std::collections::HashMap;

const CURRENT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub username: String,
    pub password: String,
    pub note: Option<String>,
    // New fields for enhanced functionality
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub url: Option<String>,
    pub totp_secret: Option<String>, // For 2FA support
}

impl Entry {
    pub fn new(username: String, password: String, note: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            username,
            password,
            note,
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            url: None,
            totp_secret: None,
        }    }
    
    #[allow(dead_code)]
    pub fn update(&mut self) {
        self.modified_at = chrono::Utc::now();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vault {
    pub version: u32,
    pub entries: HashMap<String, Entry>,
    pub metadata: VaultMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
}

impl Vault {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            version: CURRENT_VERSION,
            entries: HashMap::new(),
            metadata: VaultMetadata {
                created_at: now,
                last_accessed: now,
                description: None,
            },
        }
    }
      #[allow(dead_code)]
    pub fn update_access_time(&mut self) {
        self.metadata.last_accessed = chrono::Utc::now();
    }

    pub fn add_entry(&mut self, id: String, entry: Entry) {
        self.entries.insert(id, entry);
    }

    pub fn get_entry(&self, id: &str) -> Option<&Entry> {
        self.entries.get(id)
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<Entry> {
        self.entries.remove(id)
    }

    pub fn list_entries(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
