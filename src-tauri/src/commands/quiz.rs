use crate::models::diagnosis::*;
use crate::models::quiz::*;
use crate::services::quiz_engine;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

pub struct DiagnosisSessions(pub Mutex<HashMap<String, DiagnosisSession>>);

#[tauri::command]
pub async fn generate_quiz(
    app: AppHandle,
    params: QuizStreamParams,
    on_event: Channel<quiz_engine::QuizStreamEvent>,
) -> Result<(), String> {
    let data_dir = crate::services::storage::get_data_dir(&app)?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = on_event.send(event);
        }
    });
    quiz_engine::generate_quiz_stream(&data_dir, &params, tx).await
}

#[tauri::command]
pub async fn submit_answer_advanced(
    app: AppHandle,
    question: String,
    user_answer: String,
    user_reasoning: String,
    note_path: String,
    on_event: Channel<quiz_engine::DiagnosisStreamEvent>,
) -> Result<String, String> {
    let data_dir = crate::services::storage::get_data_dir(&app)?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = on_event.send(event);
        }
    });

    let session_id = uuid::Uuid::new_v4().to_string();
    let initial_round = quiz_engine::submit_diagnosis_initial(
        &data_dir, &question, &user_answer, &user_reasoning, &note_path, tx
    ).await?;

    let note_content = crate::services::fs_service::read_file_content(&note_path)?;
    let settings = crate::services::config::get_settings_path(&data_dir)?;
    let session = DiagnosisSession {
        session_id: session_id.clone(),
        question,
        user_answer,
        user_reasoning,
        note_path,
        note_content: note_content.chars().take(8000).collect(),
        conversation: vec![initial_round],
        current_round: 0,
        max_rounds: settings.quiz.advanced.max_diagnosis_rounds,
        final_report: None,
    };

    let sessions = app.state::<DiagnosisSessions>();
    sessions.0.lock().unwrap().insert(session_id.clone(), session);

    Ok(session_id)
}

#[tauri::command]
pub async fn diagnose_follow_up(
    app: AppHandle,
    session_id: String,
    user_reply: String,
    on_event: Channel<quiz_engine::DiagnosisStreamEvent>,
) -> Result<(), String> {
    let data_dir = crate::services::storage::get_data_dir(&app)?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = on_event.send(event);
        }
    });

    let sessions = app.state::<DiagnosisSessions>();
    let mut session = {
        let mut sessions_lock = sessions.0.lock().unwrap();
        sessions_lock.remove(&session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?
    };

    quiz_engine::diagnose_follow_up(&data_dir, &mut session, &user_reply, tx).await?;

    sessions.0.lock().unwrap().insert(session_id, session);
    Ok(())
}

#[tauri::command]
pub async fn generate_diagnosis_report(
    app: AppHandle,
    session_id: String,
) -> Result<DiagnosisReport, String> {
    let data_dir = crate::services::storage::get_data_dir(&app)?;
    let sessions = app.state::<DiagnosisSessions>();
    let session = {
        let sessions_lock = sessions.0.lock().unwrap();
        sessions_lock.get(&session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?
            .clone()
    };

    if let Some(ref report) = session.final_report {
        Ok(report.clone())
    } else {
        quiz_engine::generate_diagnosis_report(&data_dir, &session).await
    }
}
