use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub username: String,
    pub password: String,
    pub note: Option<String>,
}

impl Entry {
    pub fn new(username: String, password: String, note: Option<String>) -> Self {
        Self {
            username,
            password,
            note,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vault {
    pub entries: HashMap<String, Entry>,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
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
