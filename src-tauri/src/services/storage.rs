use serde::{de::DeserializeOwned, Serialize};
use tauri::{AppHandle, Manager};
use std::fs;
use std::path::{Path, PathBuf};

const APP_DATA_SUBDIR: &str = "knowtequiz";

pub fn get_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let dir = data_dir.join(APP_DATA_SUBDIR);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create data dir: {}", e))?;
    Ok(dir)
}

pub fn get_data_dir_path(base: &Path) -> Result<PathBuf, String> {
    let dir = base.join(APP_DATA_SUBDIR);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create data dir: {}", e))?;
    Ok(dir)
}

pub fn read_json_path<T: DeserializeOwned>(data_dir: &Path, filename: &str) -> Result<T, String> {
    let path = data_dir.join(filename);
    if !path.exists() {
        return Err(format!("File not found: {}", filename));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", filename, e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", filename, e))
}

pub fn write_json_path<T: Serialize>(data_dir: &Path, filename: &str, data: &T) -> Result<(), String> {
    let path = data_dir.join(filename);
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize {}: {}", filename, e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write {}: {}", filename, e))
}

pub fn file_exists_path(data_dir: &Path, filename: &str) -> Result<bool, String> {
    Ok(data_dir.join(filename).exists())
}

// Backward-compatible wrappers for Tauri commands
pub fn read_json<T: DeserializeOwned>(app: &AppHandle, filename: &str) -> Result<T, String> {
    let dir = get_data_dir(app)?;
    read_json_path(&dir, filename)
}

pub fn write_json<T: Serialize>(app: &AppHandle, filename: &str, data: &T) -> Result<(), String> {
    let dir = get_data_dir(app)?;
    write_json_path(&dir, filename, data)
}

pub fn file_exists(app: &AppHandle, filename: &str) -> Result<bool, String> {
    let dir = get_data_dir(app)?;
    file_exists_path(&dir, filename)
}
