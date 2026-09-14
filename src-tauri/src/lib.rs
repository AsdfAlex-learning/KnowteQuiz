mod commands;
pub mod errors;
mod models;
mod services;
mod utils;
mod web_server;

use commands::{note, quiz, settings};
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
            quiz::generate_quiz,
            quiz::submit_answer_advanced,
            quiz::diagnose_follow_up,
            quiz::generate_diagnosis_report,
            quiz::cleanup_sessions,
            settings::get_settings,
            settings::save_settings,
            settings::list_prompt_templates,
            settings::test_connection,
            settings::probe_llm,
            settings::backup_data,
            settings::get_data_status,
            settings::restore_latest_backup,
            settings::save_mistake,
            settings::load_mistakes,
            settings::mark_mistake_reviewed,
            settings::open_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running KnowteQuiz");
}

pub fn run_web_server(port: u16) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(web_server::start(port)) {
        eprintln!("Web server failed: {}", e);
        std::process::exit(1);
    }
}

pub fn run_both(port: u16) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        if let Err(e) = rt.block_on(web_server::start(port)) {
            eprintln!("Web server failed: {}", e);
            std::process::exit(1);
        }
    });
    run_desktop();
}
