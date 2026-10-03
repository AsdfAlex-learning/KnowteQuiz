use crate::errors::AppError;
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const APP_DATA_SUBDIR: &str = "knowtequiz";
/// Tauri bundle identifier; `app_data_dir()` resolves to
/// `<data_dir>/<identifier>`. Keep in sync with `tauri.conf.json`.
pub const APP_IDENTIFIER: &str = "com.knowtequiz.app";
const MANAGED_DATA_FILES: [&str; 8] = [
    "settings.json",
    "settings.json.bak",
    "mistakes.jsonl",
    "mistakes.jsonl.bak",
    "mistakes.json",
    "mistakes.json.bak",
    "index.json",
    "index.json.bak",
];

#[derive(Debug, Clone, Serialize)]
pub struct DataBackupResult {
    pub backup_dir: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataRestoreResult {
    pub backup_dir: String,
    pub pre_restore_backup_dir: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataFileStatus {
    pub name: String,
    pub exists: bool,
    pub size_bytes: u64,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataStatus {
    pub data_dir: String,
    pub files: Vec<DataFileStatus>,
}

pub fn get_data_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get app data dir: {}", e)))?;
    let dir = data_dir.join(APP_DATA_SUBDIR);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Mirrors Tauri's `app_data_dir()` (`<data_dir>/<identifier>`) so the web
/// server resolves the same data directory without a Tauri runtime.
pub fn default_app_data_dir() -> Result<PathBuf, AppError> {
    dirs::data_dir()
        .map(|base| base.join(APP_IDENTIFIER).join(APP_DATA_SUBDIR))
        .ok_or_else(|| AppError::Internal("Failed to get data directory".to_string()))
}

/// Copy managed data files and session files from the legacy web-only data dir
/// (`<data_dir>/knowtequiz`) into `target_dir`, skipping files that already
/// exist there. Returns the number of files copied; safe to run repeatedly.
pub fn migrate_legacy_web_data(legacy_dir: &Path, target_dir: &Path) -> Result<usize, AppError> {
    if !legacy_dir.exists() || legacy_dir == target_dir {
        return Ok(0);
    }

    let mut copied = 0;
    for filename in MANAGED_DATA_FILES {
        let source = legacy_dir.join(filename);
        let target = target_dir.join(filename);
        if source.is_file() && !target.exists() {
            fs::create_dir_all(target_dir)?;
            fs::copy(&source, &target)?;
            copied += 1;
        }
    }

    let legacy_sessions = legacy_dir.join("sessions");
    if legacy_sessions.is_dir() {
        let target_sessions = target_dir.join("sessions");
        for entry in fs::read_dir(&legacy_sessions)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let target_file = target_sessions.join(entry.file_name());
            if !target_file.exists() {
                fs::create_dir_all(&target_sessions)?;
                fs::copy(entry.path(), &target_file)?;
                copied += 1;
            }
        }
    }

    Ok(copied)
}

pub fn read_json_path<T: DeserializeOwned>(data_dir: &Path, filename: &str) -> Result<T, AppError> {
    let path = data_dir.join(filename);
    if !path.exists() {
        return read_json_backup_path(data_dir, filename)
            .ok_or_else(|| AppError::NotFound(format!("File not found: {}", filename)));
    }
    let content = fs::read_to_string(&path)?;
    match serde_json::from_str(&content) {
        Ok(data) => Ok(data),
        Err(parse_error) => read_json_backup_path(data_dir, filename)
            .ok_or_else(|| AppError::Internal(format!("Failed to parse {}: {}", filename, parse_error))),
    }
}

/// Serializes atomic JSON writes. The `.tmp` path is fixed per filename, so
/// two concurrent writers of the same file would otherwise interleave and
/// corrupt the tmp→rename sequence. (`mistakes.jsonl` has its own file lock.)
static JSON_WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn write_json_path<T: Serialize>(
    data_dir: &Path,
    filename: &str,
    data: &T,
) -> Result<(), AppError> {
    let _guard = JSON_WRITE_LOCK
        .lock()
        .map_err(|_| AppError::Internal("JSON write lock poisoned".to_string()))?;

    let path = data_dir.join(filename);
    let tmp_path = data_dir.join(format!("{}.tmp", filename));
    let backup_path = data_dir.join(format!("{}.bak", filename));
    let content = serde_json::to_string_pretty(data)?;

    {
        let mut tmp_file = fs::File::create(&tmp_path)?;
        use std::io::Write;
        tmp_file.write_all(content.as_bytes())?;
        tmp_file.sync_all()?;
    }

    if path.exists() {
        fs::copy(&path, &backup_path)?;
    }

    fs::rename(&tmp_path, &path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        AppError::Internal(format!("Failed to replace {}: {}", filename, e))
    })
}

fn read_json_backup_path<T: DeserializeOwned>(data_dir: &Path, filename: &str) -> Option<T> {
    let backup_path = data_dir.join(format!("{}.bak", filename));
    let content = fs::read_to_string(backup_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn backup_data_files_path(data_dir: &Path) -> Result<DataBackupResult, AppError> {
    fs::create_dir_all(data_dir)?;
    let backup_dir = data_dir.join("backups").join(format!(
        "{}-{}",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    fs::create_dir_all(&backup_dir)?;

    let mut files = Vec::new();
    for filename in MANAGED_DATA_FILES {
        let source = data_dir.join(filename);
        if source.exists() {
            fs::copy(&source, backup_dir.join(filename))?;
            files.push(filename.to_string());
        }
    }
    files.sort();

    Ok(DataBackupResult {
        backup_dir: backup_dir.to_string_lossy().to_string(),
        files,
    })
}

pub fn backup_data_files(app: &AppHandle) -> Result<DataBackupResult, AppError> {
    let dir = get_data_dir(app)?;
    backup_data_files_path(&dir)
}

pub fn restore_latest_backup_path(data_dir: &Path) -> Result<DataRestoreResult, AppError> {
    let backup_dir = latest_backup_dir(data_dir)?;
    let pre_restore = backup_data_files_path(data_dir)?;

    let mut files = Vec::new();
    for filename in MANAGED_DATA_FILES {
        let source = backup_dir.join(filename);
        if source.exists() {
            fs::copy(&source, data_dir.join(filename))?;
            files.push(filename.to_string());
        }
    }
    files.sort();

    if files.is_empty() {
        return Err(AppError::NotFound(format!(
            "No managed data files found in backup: {}",
            backup_dir.to_string_lossy()
        )));
    }

    Ok(DataRestoreResult {
        backup_dir: backup_dir.to_string_lossy().to_string(),
        pre_restore_backup_dir: pre_restore.backup_dir,
        files,
    })
}

pub fn restore_latest_backup(app: &AppHandle) -> Result<DataRestoreResult, AppError> {
    let dir = get_data_dir(app)?;
    restore_latest_backup_path(&dir)
}

fn latest_backup_dir(data_dir: &Path) -> Result<PathBuf, AppError> {
    let backups_dir = data_dir.join("backups");
    let mut candidates = Vec::new();
    if !backups_dir.exists() {
        return Err(AppError::NotFound("No backup directory found".to_string()));
    }

    for entry in fs::read_dir(&backups_dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            candidates.push(entry.path());
        }
    }

    candidates.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
    candidates
        .pop()
        .ok_or_else(|| AppError::NotFound("No backup snapshots found".to_string()))
}

pub fn data_status_path(data_dir: &Path) -> Result<DataStatus, AppError> {
    fs::create_dir_all(data_dir)?;
    let mut files = Vec::new();

    for filename in MANAGED_DATA_FILES {
        let path = data_dir.join(filename);
        if path.exists() {
            let metadata = fs::metadata(&path)?;
            let modified_at = metadata
                .modified()
                .ok()
                .map(chrono::DateTime::<chrono::Utc>::from);
            files.push(DataFileStatus {
                name: filename.to_string(),
                exists: true,
                size_bytes: metadata.len(),
                modified_at,
            });
        } else {
            files.push(DataFileStatus {
                name: filename.to_string(),
                exists: false,
                size_bytes: 0,
                modified_at: None,
            });
        }
    }

    Ok(DataStatus {
        data_dir: data_dir.to_string_lossy().to_string(),
        files,
    })
}

pub fn data_status(app: &AppHandle) -> Result<DataStatus, AppError> {
    let dir = get_data_dir(app)?;
    data_status_path(&dir)
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
    fn write_json_path_serializes_concurrent_writers() {
        let dir = temp_data_dir("write_json_path_serializes_concurrent_writers");
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let dir = dir.clone();
                std::thread::spawn(move || {
                    let record = TestRecord {
                        version: "1".to_string(),
                        value: format!("writer-{i}"),
                    };
                    write_json_path(&dir, "settings.json", &record)
                        .expect("concurrent write should succeed");
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("writer thread should not panic");
        }

        let final_record: TestRecord =
            read_json_path(&dir, "settings.json").expect("final file should parse");
        assert!(final_record.value.starts_with("writer-"));
        assert!(!dir.join("settings.json.tmp").exists());
        assert!(
            dir.join("settings.json.bak").exists(),
            "last replaced file should remain as backup"
        );
    }

    #[test]
    fn migrate_legacy_web_data_copies_missing_files_only() {
        let legacy = temp_data_dir("migrate_legacy_web_data_legacy");
        let target = temp_data_dir("migrate_legacy_web_data_target");
        fs::write(legacy.join("settings.json"), r#"{"legacy":true}"#)
            .expect("legacy settings should be written");
        fs::write(legacy.join("mistakes.jsonl"), "{}\n").expect("legacy mistakes should be written");
        fs::create_dir_all(legacy.join("sessions")).expect("legacy sessions dir");
        fs::write(legacy.join("sessions").join("s1.json"), "{}")
            .expect("legacy session should be written");
        fs::write(target.join("settings.json"), r#"{"current":true}"#)
            .expect("current settings should be written");

        let copied = migrate_legacy_web_data(&legacy, &target).expect("migration should succeed");

        assert_eq!(copied, 2);
        assert_eq!(
            fs::read_to_string(target.join("settings.json")).expect("target settings intact"),
            r#"{"current":true}"#
        );
        assert_eq!(
            fs::read_to_string(target.join("mistakes.jsonl")).expect("mistakes copied"),
            "{}\n"
        );
        assert_eq!(
            fs::read_to_string(target.join("sessions").join("s1.json")).expect("session copied"),
            "{}"
        );

        let copied_again =
            migrate_legacy_web_data(&legacy, &target).expect("second migration should succeed");
        assert_eq!(copied_again, 0);
    }

    #[test]
    fn migrate_legacy_web_data_is_noop_without_legacy_dir() {
        let target = temp_data_dir("migrate_noop_target");
        let legacy = target.join("does-not-exist");

        let copied = migrate_legacy_web_data(&legacy, &target).expect("migration should succeed");

        assert_eq!(copied, 0);
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

    #[test]
    fn backup_data_files_path_copies_existing_json_files_to_backup_dir() {
        let dir = temp_data_dir("backup_data_files_path_copies_existing_json_files_to_backup_dir");
        fs::write(dir.join("settings.json"), r#"{"theme":"dark"}"#)
            .expect("settings should be written");
        fs::write(dir.join("mistakes.json"), r#"[{"id":"m1"}]"#)
            .expect("mistakes should be written");
        fs::write(dir.join("index.json"), r#"{"notes":[]}"#).expect("index should be written");
        fs::write(dir.join("scratch.tmp"), "temporary").expect("unmanaged file should be written");

        let result = backup_data_files_path(&dir).expect("backup should succeed");

        assert!(result.backup_dir.contains("backups"));
        assert_eq!(
            result.files,
            vec!["index.json", "mistakes.json", "settings.json"]
        );
        let backup_dir = PathBuf::from(&result.backup_dir);
        assert_eq!(
            fs::read_to_string(backup_dir.join("settings.json"))
                .expect("settings backup should exist"),
            r#"{"theme":"dark"}"#
        );
        assert_eq!(
            fs::read_to_string(backup_dir.join("mistakes.json"))
                .expect("mistakes backup should exist"),
            r#"[{"id":"m1"}]"#
        );
        assert_eq!(
            fs::read_to_string(backup_dir.join("index.json"))
                .expect("index backup should exist"),
            r#"{"notes":[]}"#
        );
        assert!(!backup_dir.join("scratch.tmp").exists());
    }

    #[test]
    fn data_status_path_reports_managed_file_sizes_and_missing_files() {
        let dir = temp_data_dir("data_status_path_reports_managed_file_sizes_and_missing_files");
        fs::write(dir.join("settings.json"), "12345").expect("settings should be written");
        fs::write(dir.join("mistakes.json"), "[]").expect("mistakes should be written");
        fs::write(dir.join("index.json"), "123").expect("index should be written");

        let status = data_status_path(&dir).expect("data status should load");

        assert_eq!(status.data_dir, dir.to_string_lossy());
        let settings = status
            .files
            .iter()
            .find(|file| file.name == "settings.json")
            .expect("settings status should exist");
        assert!(settings.exists);
        assert_eq!(settings.size_bytes, 5);
        assert!(settings.modified_at.is_some());

        let settings_backup = status
            .files
            .iter()
            .find(|file| file.name == "settings.json.bak")
            .expect("settings backup status should exist");
        assert!(!settings_backup.exists);
        assert_eq!(settings_backup.size_bytes, 0);
        assert!(settings_backup.modified_at.is_none());

        let index = status
            .files
            .iter()
            .find(|file| file.name == "index.json")
            .expect("index status should exist");
        assert!(index.exists);
        assert_eq!(index.size_bytes, 3);
        assert!(index.modified_at.is_some());
    }

    #[test]
    fn restore_latest_backup_path_restores_newest_backup_and_preserves_current_files() {
        let dir = temp_data_dir(
            "restore_latest_backup_path_restores_newest_backup_and_preserves_current_files",
        );
        fs::write(dir.join("settings.json"), "current-settings")
            .expect("current settings should be written");
        fs::write(dir.join("mistakes.json"), "current-mistakes")
            .expect("current mistakes should be written");

        let older = dir.join("backups").join("20260101-010000-old");
        let newer = dir.join("backups").join("20260201-010000-new");
        fs::create_dir_all(&older).expect("older backup should be created");
        fs::create_dir_all(&newer).expect("newer backup should be created");
        fs::write(older.join("settings.json"), "old-settings")
            .expect("older settings backup should be written");
        fs::write(newer.join("settings.json"), "new-settings")
            .expect("newer settings backup should be written");
        fs::write(newer.join("mistakes.json"), "new-mistakes")
            .expect("newer mistakes backup should be written");

        let result = restore_latest_backup_path(&dir).expect("latest backup should restore");

        assert_eq!(result.backup_dir, newer.to_string_lossy());
        assert_eq!(result.files, vec!["mistakes.json", "settings.json"]);
        assert_eq!(
            fs::read_to_string(dir.join("settings.json")).expect("settings should be restored"),
            "new-settings"
        );
        assert_eq!(
            fs::read_to_string(dir.join("mistakes.json")).expect("mistakes should be restored"),
            "new-mistakes"
        );

        let pre_restore_dir = PathBuf::from(result.pre_restore_backup_dir);
        assert_eq!(
            fs::read_to_string(pre_restore_dir.join("settings.json"))
                .expect("pre-restore settings backup should exist"),
            "current-settings"
        );
        assert_eq!(
            fs::read_to_string(pre_restore_dir.join("mistakes.json"))
                .expect("pre-restore mistakes backup should exist"),
            "current-mistakes"
        );
    }
}
