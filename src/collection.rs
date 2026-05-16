use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct HeaderPair {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SavedRequest {
    pub name: String,
    pub url: String,
    pub method: String,
    pub headers: Vec<HeaderPair>,
    pub body: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Collection {
    pub requests: Vec<SavedRequest>,
}

pub fn load_collection(path: &str) -> Collection {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(collection) = serde_json::from_str(&content) {
            return collection;
        }
    }
    Collection::default()
}

pub fn save_collection(path: &str, collection: &Collection) -> Result<(), String> {
    let content = serde_json::to_string_pretty(collection)
        .map_err(|e| format!("Erro ao serializar coleção: {}", e))?;
    fs::write(path, content)
        .map_err(|e| format!("Erro ao salvar arquivo: {}", e))?;
    Ok(())
}
