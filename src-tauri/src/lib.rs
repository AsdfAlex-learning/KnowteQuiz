mod commands;
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
            settings::get_settings,
            settings::save_settings,
            settings::list_prompt_templates,
            settings::test_connection,
            settings::backup_data,
            settings::get_data_status,
            settings::restore_latest_backup,
            settings::save_mistake,
            settings::load_mistakes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running KnowteQuiz");
}

pub fn run_web_server(port: u16) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        web_server::start(port).await;
    });
}

pub fn run_both(port: u16) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            web_server::start(port).await;
        });
    });
    run_desktop();
}
