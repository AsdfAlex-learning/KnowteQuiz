mod commands;
pub mod errors;
mod models;
mod services;
mod utils;
mod web_server;

use commands::{doc_convert, exploration, llm_run, note, quiz, settings};
use quiz::DiagnosisSessions;
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_desktop() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DiagnosisSessions(Mutex::new(HashMap::new())))
        .invoke_handler(tauri::generate_handler![
            note::select_folder,
            note::scan_notes,
            note::read_note,
            note::list_directories,
            quiz::generate_quiz,
            quiz::submit_answer_advanced,
            quiz::diagnose_follow_up,
            quiz::generate_diagnosis_report,
            quiz::cleanup_sessions,
            doc_convert::convert_document,
            doc_convert::get_document_status,
            exploration::exploration_create,
            exploration::exploration_list,
            exploration::exploration_get,
            exploration::exploration_delete,
            exploration::exploration_start_stream,
            exploration::exploration_confirm_domains,
            exploration::exploration_next_question,
            exploration::exploration_ask,
            exploration::exploration_undo,
            exploration::exploration_redo,
            exploration::exploration_finish,
            exploration::exploration_skip,
            exploration::exploration_aside,
            exploration::exploration_export,
            exploration::exploration_save_to_vault,
            exploration::exploration_import,
            exploration::exploration_continue,
            llm_run::llm_list_runs,
            llm_run::llm_get_run,
            llm_run::llm_get_run_content,
            settings::get_settings,
            settings::save_settings,
            settings::list_prompt_templates,
            settings::test_connection,
            settings::probe_llm,
            settings::list_models,
            settings::backup_data,
            settings::get_data_status,
            settings::restore_latest_backup,
            settings::save_mistake,
            settings::load_mistakes,
            settings::mark_mistake_reviewed,
            settings::open_data_dir,
            settings::save_media_file,
        ])
        .setup(|app| {
            // Best-effort cleanup of stale diagnosis sessions on startup.
            if let Ok(data_dir) = crate::services::storage::get_data_dir(app.handle()) {
                std::thread::spawn(move || {
                    let _ = crate::services::diagnosis_session_service::cleanup_expired_sessions(
                        &data_dir, 7,
                    );
                    let _ = crate::services::exploration::session_store::cleanup_expired_sessions(
                        &data_dir, 7,
                    );
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running KnowteQuiz");
}

pub fn run_web_server(port: u16, data_dir_override: Option<String>) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(web_server::start(
        port,
        data_dir_override.map(std::path::PathBuf::from),
    )) {
        eprintln!("Web server failed: {}", e);
        std::process::exit(1);
    }
}

pub fn run_both(port: u16, data_dir_override: Option<String>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        if let Err(e) = rt.block_on(web_server::start(
            port,
            data_dir_override.map(std::path::PathBuf::from),
        )) {
            eprintln!("Web server failed: {}", e);
            std::process::exit(1);
        }
    });
    run_desktop();
}
