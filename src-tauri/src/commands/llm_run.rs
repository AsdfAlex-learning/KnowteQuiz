use crate::errors::AppError;
use crate::models::llm_run::{LlmRunRecord, LlmRunSummary};
use crate::services::{llm_run_store, storage};
use tauri::AppHandle;

/// List recorded LLM runs, newest first. Metadata only — use
/// `llm_get_run_content` for the prompt / raw response bodies.
#[tauri::command]
pub async fn llm_list_runs(
    app: AppHandle,
    limit: Option<usize>,
    kind: Option<String>,
) -> Result<Vec<LlmRunSummary>, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    tokio::task::spawn_blocking(move || {
        llm_run_store::list_llm_runs(&data_dir, limit.unwrap_or(50), kind.as_deref())
    })
    .await
    .map_err(|e| AppError::Internal(format!("LLM run list task failed: {e}")))?
}

/// Full record for a single run, including prompt/response file references.
#[tauri::command]
pub async fn llm_get_run(app: AppHandle, id: String) -> Result<LlmRunRecord, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    tokio::task::spawn_blocking(move || llm_run_store::get_llm_run(&data_dir, &id))
        .await
        .map_err(|e| AppError::Internal(format!("LLM run lookup task failed: {e}")))?
}

/// Read the `prompt` or `response` text of a run. `field` must be
/// `"prompt"` or `"response"`. Returns an empty string when that content was
/// not recorded (e.g. `record_llm_io = false`).
#[tauri::command]
pub async fn llm_get_run_content(
    app: AppHandle,
    id: String,
    field: String,
) -> Result<String, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    tokio::task::spawn_blocking(move || {
        let run = llm_run_store::get_llm_run(&data_dir, &id)?;
        let filename = match field.as_str() {
            "prompt" => run.prompt_file,
            "response" => run.response_file,
            other => {
                return Err(AppError::InvalidInput(format!(
                    "Unknown run content field: {other}"
                )))
            }
        };
        match filename {
            Some(name) => llm_run_store::read_run_content(&data_dir, &name),
            None => Ok(String::new()),
        }
    })
    .await
    .map_err(|e| AppError::Internal(format!("LLM run content task failed: {e}")))?
}
