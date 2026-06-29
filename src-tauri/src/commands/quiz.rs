use crate::models::diagnosis::*;
use crate::models::quiz::*;
use crate::services::{diagnosis_session_service, quiz_engine};
use diagnosis_session_service::SessionCleanupResult;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

pub struct DiagnosisSessions(pub Mutex<HashMap<String, DiagnosisSession>>);

fn load_diagnosis_session(
    app: &AppHandle,
    data_dir: &Path,
    session_id: &str,
) -> Result<DiagnosisSession, String> {
    let sessions = app.state::<DiagnosisSessions>();
    if let Ok(mut sessions_lock) = sessions.0.lock() {
        if let Some(session) = sessions_lock.remove(session_id) {
            return Ok(session);
        }
    }
    diagnosis_session_service::load_session(data_dir, session_id)
}

fn cache_diagnosis_session(
    app: &AppHandle,
    data_dir: &Path,
    session: DiagnosisSession,
) -> Result<(), String> {
    diagnosis_session_service::save_session(data_dir, &session)?;
    let sessions = app.state::<DiagnosisSessions>();
    sessions
        .0
        .lock()
        .map_err(|_| "Failed to lock diagnosis sessions".to_string())?
        .insert(session.session_id.clone(), session);
    Ok(())
}

fn finish_diagnosis_session(
    app: &AppHandle,
    data_dir: &Path,
    session: DiagnosisSession,
) -> Result<(), String> {
    if session.final_report.is_some() {
        diagnosis_session_service::delete_session(data_dir, &session.session_id)?;
        let sessions = app.state::<DiagnosisSessions>();
        sessions
            .0
            .lock()
            .map_err(|_| "Failed to lock diagnosis sessions".to_string())?
            .remove(&session.session_id);
        Ok(())
    } else {
        cache_diagnosis_session(app, data_dir, session)
    }
}

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
    correct_answer: String,
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
        &data_dir,
        &question,
        &correct_answer,
        &user_answer,
        &user_reasoning,
        &note_path,
        tx,
    )
    .await?;

    let note_content = crate::services::fs_service::read_file_content(&note_path)?;
    let note_body = crate::services::note_service::extract_body_content(&note_content);
    let settings = crate::services::config::get_settings_path(&data_dir)?;
    let session = DiagnosisSession {
        session_id: session_id.clone(),
        question,
        user_answer,
        user_reasoning,
        note_path,
        note_content: note_body.chars().take(8000).collect(),
        conversation: vec![initial_round],
        current_round: 0,
        max_rounds: settings.quiz.advanced.max_diagnosis_rounds,
        final_report: None,
    };

    cache_diagnosis_session(&app, &data_dir, session)?;

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

    let mut session = load_diagnosis_session(&app, &data_dir, &session_id)
        .map_err(|_| format!("Session {} not found", session_id))?;

    if let Err(err) =
        quiz_engine::diagnose_follow_up(&data_dir, &mut session, &user_reply, tx).await
    {
        cache_diagnosis_session(&app, &data_dir, session)?;
        return Err(err);
    }

    finish_diagnosis_session(&app, &data_dir, session)?;
    Ok(())
}

#[tauri::command]
pub async fn generate_diagnosis_report(
    app: AppHandle,
    session_id: String,
) -> Result<DiagnosisReport, String> {
    let data_dir = crate::services::storage::get_data_dir(&app)?;
    let mut session = load_diagnosis_session(&app, &data_dir, &session_id)
        .map_err(|_| format!("Session {} not found", session_id))?;

    if let Some(ref report) = session.final_report {
        finish_diagnosis_session(&app, &data_dir, session.clone())?;
        Ok(report.clone())
    } else {
        let report = quiz_engine::generate_diagnosis_report(&data_dir, &session).await?;
        session.final_report = Some(report.clone());
        finish_diagnosis_session(&app, &data_dir, session)?;
        Ok(report)
    }
}

#[tauri::command]
pub async fn cleanup_sessions(app: AppHandle) -> Result<SessionCleanupResult, String> {
    let data_dir = crate::services::storage::get_data_dir(&app)?;
    diagnosis_session_service::cleanup_expired_sessions(&data_dir, 7)
}
