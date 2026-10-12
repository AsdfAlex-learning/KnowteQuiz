use crate::errors::AppError;
use crate::models::llm_run::{LlmRunRecord, LlmRunSummary};
use std::io::Write as _;
use std::path::Path;
use std::sync::Mutex;

const RUNS_DIR: &str = "llm_runs";
const RUNS_FILE: &str = "llm_runs.jsonl";
const DEBUG_DIR: &str = "debug";
/// Max records retained in `llm_runs.jsonl` (newest kept).
const RUNS_RETENTION: usize = 500;
/// Max raw-response/prompt files retained under `debug/` (oldest pruned).
const DEBUG_LOG_RETENTION: usize = 100;

/// Serializes appends to `llm_runs.jsonl`.
///
/// Appending is a read-modify-write on the whole file when rolling, so two
/// concurrent recorders must not interleave. Process-local lock (mirrors
/// `MISTAKES_WRITE_LOCK` in `mistake_service.rs`).
static RUNS_WRITE_LOCK: Mutex<()> = Mutex::new(());

fn lock_runs_write() -> Result<std::sync::MutexGuard<'static, ()>, AppError> {
    RUNS_WRITE_LOCK
        .lock()
        .map_err(|_| AppError::Internal("LLM run write lock poisoned".to_string()))
}

fn runs_dir(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(RUNS_DIR)
}

fn runs_file(data_dir: &Path) -> std::path::PathBuf {
    runs_dir(data_dir).join(RUNS_FILE)
}

/// Append a run record to `llm_runs/llm_runs.jsonl`, then roll the file to the
/// newest `RUNS_RETENTION` lines.
pub fn record_llm_run(data_dir: &Path, record: &LlmRunRecord) -> Result<(), AppError> {
    let _guard = lock_runs_write()?;
    let dir = runs_dir(data_dir);
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Internal(format!("Failed to create {RUNS_DIR} dir: {e}")))?;

    let line = serde_json::to_string(record)
        .map_err(|e| AppError::Internal(format!("Failed to serialize LLM run: {e}")))?;

    let path = runs_file(data_dir);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| AppError::Internal(format!("Failed to open {}: {e}", path.display())))?;
    writeln!(file, "{line}")
        .map_err(|e| AppError::Internal(format!("Failed to append to {}: {e}", path.display())))?;
    drop(file);

    roll_runs_file(&path);
    Ok(())
}

/// Rewrite `llm_runs.jsonl` keeping only the newest `RUNS_RETENTION` records.
/// Best effort: a failure to roll is logged-in-place but never fails the write.
fn roll_runs_file(path: &Path) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let mut lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if lines.len() <= RUNS_RETENTION {
        return;
    }
    let excess = lines.len() - RUNS_RETENTION;
    lines.drain(0..excess);
    let rolled: String = lines.join("\n") + "\n";
    let tmp = path.with_extension("jsonl.tmp");
    if std::fs::write(&tmp, rolled).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    } else {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Write a free-form debug text file under `debug/` and prune to the retention
/// cap. Returns the bare filename (not the path) on success.
///
/// `suffix` becomes the filename suffix (e.g. `"raw"` or `"prompt"`), and `kind`
/// the semantic tag (e.g. `"quiz"`), producing
/// `{timestamp}_{uuid}_{kind}_{suffix}.txt`.
pub fn write_debug_file(
    data_dir: &Path,
    kind: &str,
    suffix: &str,
    content: &str,
) -> Option<String> {
    let debug_dir = data_dir.join(DEBUG_DIR);
    std::fs::create_dir_all(&debug_dir).ok()?;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    // uuid suffix: concurrent calls within the same second must not overwrite.
    let filename = format!(
        "{timestamp}_{}_{}_{}.txt",
        uuid::Uuid::new_v4().simple(),
        kind,
        suffix
    );

    std::fs::write(debug_dir.join(&filename), content).ok()?;
    prune_debug_logs(&debug_dir);
    Some(filename)
}

/// Remove the oldest debug files beyond the retention cap. Best effort.
pub fn prune_debug_logs(debug_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(debug_dir) else {
        return;
    };
    let mut files: Vec<(std::time::SystemTime, std::path::PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((modified, entry.path()))
        })
        .collect();
    if files.len() <= DEBUG_LOG_RETENTION {
        return;
    }
    files.sort_by_key(|(modified, _)| *modified);
    let excess = files.len() - DEBUG_LOG_RETENTION;
    for (_, path) in files.into_iter().take(excess) {
        let _ = std::fs::remove_file(path);
    }
}

fn read_all_runs(data_dir: &Path) -> Result<Vec<LlmRunRecord>, AppError> {
    let path = runs_file(data_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Internal(format!("Failed to read {}: {e}", path.display())))?;
    Ok(content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<LlmRunRecord>(line).ok())
        .collect())
}

/// Return the newest runs, newest first, optionally filtered by kind.
pub fn list_llm_runs(
    data_dir: &Path,
    limit: usize,
    kind: Option<&str>,
) -> Result<Vec<LlmRunSummary>, AppError> {
    let mut runs = read_all_runs(data_dir)?;
    runs.reverse(); // file is append-ordered; reverse so newest is first
    if let Some(kind) = kind {
        runs.retain(|r| r.kind.as_str() == kind);
    }
    let limit = limit.max(1);
    Ok(runs.iter().take(limit).map(LlmRunSummary::from).collect())
}

/// Find a single run by id. `AppError::NotFound` when absent.
pub fn get_llm_run(data_dir: &Path, id: &str) -> Result<LlmRunRecord, AppError> {
    read_all_runs(data_dir)?
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound(format!("LLM run {id} not found")))
}

/// Read a `debug/` content file referenced by a run record.
///
/// `filename` must be a bare filename; any path separator or `..` is rejected
/// (this is the path-traversal guard).
pub fn read_run_content(data_dir: &Path, filename: &str) -> Result<String, AppError> {
    if filename.trim().is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
    {
        return Err(AppError::InvalidInput(format!(
            "Invalid run content filename: {filename}"
        )));
    }
    let path = data_dir.join(DEBUG_DIR).join(filename);
    std::fs::read_to_string(&path)
        .map_err(|e| AppError::Internal(format!("Failed to read {}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::llm_run::{LlmRunKind, LlmUsageSource};
    use std::path::PathBuf;

    fn temp_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-llm-run-store-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    fn record(id: &str, kind: LlmRunKind) -> LlmRunRecord {
        LlmRunRecord {
            id: id.to_string(),
            kind,
            ts_start: "2026-10-12T12:00:00.000Z".to_string(),
            ts_end: "2026-10-12T12:00:01.000Z".to_string(),
            duration_ms: 1000,
            model: "m".to_string(),
            endpoint_host: "localhost:11434".to_string(),
            request_params: serde_json::json!({ "temperature": 0.3 }),
            usage: None,
            usage_source: LlmUsageSource::Unavailable,
            http_status: Some(200),
            error: None,
            prompt_file: None,
            response_file: None,
        }
    }

    #[test]
    fn record_then_read_back_round_trips() {
        let dir = temp_dir("record_then_read_back_round_trips");
        record_llm_run(&dir, &record("id-1", LlmRunKind::Quiz)).expect("record should succeed");

        let runs = list_llm_runs(&dir, 10, None).expect("list should succeed");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].id, "id-1");
        assert_eq!(runs[0].kind, LlmRunKind::Quiz);

        let one = get_llm_run(&dir, "id-1").expect("get should succeed");
        assert_eq!(one.id, "id-1");
    }

    #[test]
    fn list_returns_newest_first_and_respects_limit_and_kind() {
        let dir = temp_dir("list_returns_newest_first_and_respects_limit_and_kind");
        for (i, kind) in [
            (1, LlmRunKind::Quiz),
            (2, LlmRunKind::DiagnosisInitial),
            (3, LlmRunKind::QuizFixup),
        ] {
            record_llm_run(&dir, &record(&format!("id-{i}"), kind)).expect("record should succeed");
        }

        let all = list_llm_runs(&dir, 10, None).expect("list should succeed");
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, "id-3", "newest first");
        assert_eq!(all[2].id, "id-1");

        let limited = list_llm_runs(&dir, 2, None).expect("list should succeed");
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].id, "id-3");

        let filtered = list_llm_runs(&dir, 10, Some("quiz")).expect("list should succeed");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "id-1");
    }

    #[test]
    fn get_returns_not_found_for_bad_id() {
        let dir = temp_dir("get_returns_not_found_for_bad_id");
        record_llm_run(&dir, &record("id-1", LlmRunKind::Quiz)).expect("record should succeed");

        let err = get_llm_run(&dir, "missing").expect_err("missing id should not resolve");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn jsonl_rolls_to_retention_limit() {
        let dir = temp_dir("jsonl_rolls_to_retention_limit");
        for i in 0..(RUNS_RETENTION + 25) {
            record_llm_run(&dir, &record(&format!("id-{i}"), LlmRunKind::Quiz))
                .expect("record should succeed");
        }

        let runs = list_llm_runs(&dir, 1000, None).expect("list should succeed");
        assert_eq!(runs.len(), RUNS_RETENTION, "rolled to retention limit");
        assert_eq!(
            runs[0].id,
            format!("id-{}", RUNS_RETENTION + 24),
            "newest record kept"
        );
        assert_eq!(runs[RUNS_RETENTION - 1].id, "id-25", "oldest retained");
    }

    #[test]
    fn read_run_content_rejects_traversal() {
        let dir = temp_dir("read_run_content_rejects_traversal");
        for bad in ["../secret.txt", "..\\secret.txt", "a/b.txt", "a\\b.txt", ""] {
            let err = read_run_content(&dir, bad).expect_err("traversal must be rejected");
            assert!(err.to_string().contains("Invalid run content filename"));
        }
    }

    #[test]
    fn read_run_content_reads_written_file() {
        let dir = temp_dir("read_run_content_reads_written_file");
        let filename =
            write_debug_file(&dir, "quiz", "raw", "hello").expect("write should succeed");
        let content = read_run_content(&dir, &filename).expect("read should succeed");
        assert_eq!(content, "hello");
    }

    #[test]
    fn write_debug_file_prunes_to_retention_cap() {
        let dir = temp_dir("write_debug_file_prunes_to_retention_cap");
        for i in 0..(DEBUG_LOG_RETENTION + 5) {
            write_debug_file(&dir, "quiz", "raw", &format!("content {i}"))
                .expect("write should succeed");
        }
        let count = std::fs::read_dir(dir.join(DEBUG_DIR))
            .expect("debug dir should exist")
            .count();
        assert!(
            count <= DEBUG_LOG_RETENTION,
            "pruning kept {count} files, expected <= {DEBUG_LOG_RETENTION}"
        );
    }

    #[test]
    fn concurrent_records_all_land() {
        let dir = temp_dir("concurrent_records_all_land");
        std::thread::scope(|scope| {
            for i in 0..16 {
                let dir = dir.clone();
                scope.spawn(move || {
                    record_llm_run(&dir, &record(&format!("id-{i}"), LlmRunKind::Quiz))
                        .expect("concurrent record should succeed");
                });
            }
        });
        let runs = list_llm_runs(&dir, 1000, None).expect("list should succeed");
        assert_eq!(runs.len(), 16, "every concurrent record must land");
    }
}
