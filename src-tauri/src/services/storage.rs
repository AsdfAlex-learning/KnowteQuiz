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
        return read_json_backup_path(data_dir, filename)
            .ok_or_else(|| format!("File not found: {}", filename))?;
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", filename, e))?;
    match serde_json::from_str(&content) {
        Ok(data) => Ok(data),
        Err(parse_error) => read_json_backup_path(data_dir, filename)
            .ok_or_else(|| format!("Failed to parse {}: {}", filename, parse_error)),
    }
}

pub fn write_json_path<T: Serialize>(data_dir: &Path, filename: &str, data: &T) -> Result<(), String> {
    let path = data_dir.join(filename);
    let tmp_path = data_dir.join(format!("{}.tmp", filename));
    let backup_path = data_dir.join(format!("{}.bak", filename));
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize {}: {}", filename, e))?;

    {
        let mut tmp_file = fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create temp file for {}: {}", filename, e))?;
        use std::io::Write;
        tmp_file
            .write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write temp file for {}: {}", filename, e))?;
        tmp_file
            .sync_all()
            .map_err(|e| format!("Failed to sync temp file for {}: {}", filename, e))?;
    }

    if path.exists() {
        fs::copy(&path, &backup_path)
            .map_err(|e| format!("Failed to backup {}: {}", filename, e))?;
    }

    fs::rename(&tmp_path, &path)
        .map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            format!("Failed to replace {}: {}", filename, e)
        })
}

fn read_json_backup_path<T: DeserializeOwned>(data_dir: &Path, filename: &str) -> Option<T> {
    let backup_path = data_dir.join(format!("{}.bak", filename));
    let content = fs::read_to_string(backup_path).ok()?;
    serde_json::from_str(&content).ok()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestRecord {
        version: String,
        value: String,
    }

    fn temp_data_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-storage-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    #[test]
    fn write_json_path_preserves_previous_file_as_backup() {
        let dir = temp_data_dir("write_json_path_preserves_previous_file_as_backup");
        let first = TestRecord {
            version: "1".to_string(),
            value: "before".to_string(),
        };
        let second = TestRecord {
            version: "1".to_string(),
            value: "after".to_string(),
        };

        write_json_path(&dir, "settings.json", &first).expect("first write should succeed");
        write_json_path(&dir, "settings.json", &second).expect("second write should succeed");

        let current: TestRecord =
            read_json_path(&dir, "settings.json").expect("current file should be readable");
        let backup: TestRecord =
            read_json_path(&dir, "settings.json.bak").expect("backup file should be readable");

        assert_eq!(current, second);
        assert_eq!(backup, first);
        assert!(!dir.join("settings.json.tmp").exists());
    }

    #[test]
    fn read_json_path_recovers_from_backup_when_primary_is_corrupt() {
        let dir = temp_data_dir("read_json_path_recovers_from_backup_when_primary_is_corrupt");
        let backup = TestRecord {
            version: "1".to_string(),
            value: "safe copy".to_string(),
        };

        write_json_path(&dir, "mistakes.json.bak", &backup).expect("backup write should succeed");
        fs::write(dir.join("mistakes.json"), "{ not valid json")
            .expect("corrupt primary should be written");

        let recovered: TestRecord =
            read_json_path(&dir, "mistakes.json").expect("backup should be used");

        assert_eq!(recovered, backup);
    }
}
