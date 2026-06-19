use axum::{
    extract::{Json, Path as AxumPath, Query, State},
    response::sse::{Event, Sse},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::models::diagnosis::{DiagnosisReport, DiagnosisSession};
use crate::models::mistake::MistakeEntry;
use crate::models::note::{NoteContent, NoteTreeNode};
use crate::models::quiz::QuizStreamParams;
use crate::models::settings::Settings;
use crate::services::{config, fs_service, note_service, quiz_engine, storage};

pub struct AppState {
    pub data_dir: PathBuf,
    pub diagnosis_sessions: Mutex<HashMap<String, DiagnosisSession>>,
}

fn get_dist_dir() -> PathBuf {
    let mut candidates = vec![PathBuf::from("../dist")];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("../../../dist"));
            candidates.push(exe_dir.join("../../dist"));
            candidates.push(exe_dir.join("dist"));
        }
    }
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }
    PathBuf::from("../dist")
}

fn listener_addr(port: u16) -> String {
    format!("127.0.0.1:{}", port)
}

fn send_missing_session_error(
    tx: &UnboundedSender<quiz_engine::DiagnosisStreamEvent>,
    session_id: &str,
) {
    let _ = tx.send(quiz_engine::DiagnosisStreamEvent::Error {
        message: format!("Session {} not found", session_id),
    });
}

pub async fn start(port: u16) {
    let data_dir = dirs::data_dir()
        .expect("Failed to get data directory")
        .join("knowtequiz");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let app_state = Arc::new(AppState {
        data_dir,
        diagnosis_sessions: Mutex::new(HashMap::new()),
    });

    let dist_dir = get_dist_dir();
    let index_path = dist_dir.join("index.html");
    let serve_dir = ServeDir::new(&dist_dir)
        .fallback(ServeFile::new(index_path));

    let app = Router::new()
        .route("/api/notes/scan", get(scan_notes_handler))
        .route("/api/notes/read", get(read_note_handler))
        .route("/api/settings", get(get_settings_handler).post(save_settings_handler))
        .route("/api/test-connection", post(test_connection_handler))
        .route("/api/prompt-templates", get(list_prompt_templates_handler))
        .route("/api/mistakes", get(load_mistakes_handler).post(save_mistake_handler))
        .route("/api/quiz/generate", post(generate_quiz_handler))
        .route("/api/quiz/diagnose", post(submit_diagnosis_handler))
        .route("/api/quiz/diagnose/{session_id}/follow_up", post(diagnose_follow_up_handler))
        .route("/api/quiz/diagnose/{session_id}/report", get(generate_report_handler))
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(listener_addr(port))
        .await
        .expect("Failed to bind TCP listener");

    println!("KnowteQuiz web server running on http://localhost:{}", port);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

#[derive(Deserialize)]
struct ScanNotesQuery {
    root_path: String,
}

async fn scan_notes_handler(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<ScanNotesQuery>,
) -> Result<Json<Vec<NoteTreeNode>>, String> {
    let result = fs_service::scan_directory(&query.root_path)?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct ReadNoteQuery {
    path: String,
}

async fn read_note_handler(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<ReadNoteQuery>,
) -> Result<Json<NoteContent>, String> {
    let content = fs_service::read_file_content(&query.path)?;
    let result = note_service::extract_metadata(&content, &query.path);
    Ok(Json(result))
}

async fn get_settings_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Settings>, String> {
    let settings = config::get_settings_path(&state.data_dir)?;
    Ok(Json(settings))
}

async fn save_settings_handler(
    State(state): State<Arc<AppState>>,
    Json(settings): Json<Settings>,
) -> Result<Json<bool>, String> {
    config::save_settings_path(&state.data_dir, &settings)?;
    Ok(Json(true))
}

async fn test_connection_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<bool>, String> {
    let settings = config::get_settings_path(&state.data_dir)?;
    let llm = &settings.llm;

    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "model": llm.model,
        "messages": [
            { "role": "user", "content": "hi" }
        ],
        "max_tokens": 5,
    });

    let response = client
        .post(format!("{}/chat/completions", llm.base_url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    Ok(Json(response.status().is_success()))
}

async fn list_prompt_templates_handler(
) -> Result<Json<Vec<(String, String, String)>>, String> {
    Ok(Json(crate::utils::prompt_templates::list_template_sets()))
}

async fn load_mistakes_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<MistakeEntry>>, String> {
    let mistakes: Vec<MistakeEntry> = storage::read_json_path(&state.data_dir, "mistakes.json")
        .unwrap_or_default();
    Ok(Json(mistakes))
}

async fn save_mistake_handler(
    State(state): State<Arc<AppState>>,
    Json(entry): Json<MistakeEntry>,
) -> Result<Json<bool>, String> {
    let mut mistakes: Vec<MistakeEntry> = storage::read_json_path(&state.data_dir, "mistakes.json")
        .unwrap_or_default();
    mistakes.insert(0, entry);
    storage::write_json_path(&state.data_dir, "mistakes.json", &mistakes)?;
    Ok(Json(true))
}

async fn generate_quiz_handler(
    State(state): State<Arc<AppState>>,
    Json(params): Json<QuizStreamParams>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let data_dir = state.data_dir.clone();
    tokio::spawn(async move {
        let _ = quiz_engine::generate_quiz_stream(&data_dir, &params, tx).await;
    });

    let stream = UnboundedReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().data(data))
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn submit_diagnosis_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let session_id = payload["session_id"].as_str().unwrap_or("").to_string();
    let question = payload["question"].as_str().unwrap_or("").to_string();
    let user_answer = payload["user_answer"].as_str().unwrap_or("").to_string();
    let user_reasoning = payload["user_reasoning"].as_str().unwrap_or("").to_string();
    let note_path = payload["note_path"].as_str().unwrap_or("").to_string();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<quiz_engine::DiagnosisStreamEvent>();
    let app_state = state.clone();

    // Create session upfront so follow_up can find it
    if let Ok(note_content) = fs_service::read_file_content(&note_path) {
        if let Ok(settings) = config::get_settings_path(&app_state.data_dir) {
            let session = DiagnosisSession {
                session_id: session_id.clone(),
                question: question.clone(),
                user_answer: user_answer.clone(),
                user_reasoning: user_reasoning.clone(),
                note_path: note_path.clone(),
                note_content: note_content.chars().take(8000).collect(),
                conversation: vec![],
                current_round: 0,
                max_rounds: settings.quiz.advanced.max_diagnosis_rounds,
                final_report: None,
            };
            if let Ok(mut sessions_lock) = app_state.diagnosis_sessions.lock() {
                sessions_lock.insert(session_id.clone(), session);
            }
        }
    }

    tokio::spawn(async move {
        if let Ok(initial_round) = quiz_engine::submit_diagnosis_initial(
            &app_state.data_dir, &question, &user_answer, &user_reasoning, &note_path, tx,
        ).await {
            if let Ok(mut sessions_lock) = app_state.diagnosis_sessions.lock() {
                if let Some(session) = sessions_lock.get_mut(&session_id) {
                    session.conversation.push(initial_round);
                }
            }
        }
    });

    let stream = UnboundedReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().data(data))
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn diagnose_follow_up_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(session_id): AxumPath<String>,
    Json(payload): Json<serde_json::Value>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let user_reply = payload["user_reply"].as_str().unwrap_or("").to_string();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<quiz_engine::DiagnosisStreamEvent>();

    let app_state = state.clone();

    tokio::spawn(async move {
        let mut session = {
            let mut sessions_lock = app_state.diagnosis_sessions.lock().unwrap();
            match sessions_lock.remove(&session_id) {
                Some(s) => s,
                None => {
                    send_missing_session_error(&tx, &session_id);
                    return;
                }
            }
        };

        let _ = quiz_engine::diagnose_follow_up(&app_state.data_dir, &mut session, &user_reply, tx).await;

        let mut sessions_lock = app_state.diagnosis_sessions.lock().unwrap();
        sessions_lock.insert(session_id, session);
    });

    let stream = UnboundedReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().data(data))
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn generate_report_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(session_id): AxumPath<String>,
) -> Result<Json<DiagnosisReport>, String> {
    let session = {
        let sessions = state.diagnosis_sessions.lock().unwrap();
        sessions
            .get(&session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?
            .clone()
    };

    if let Some(ref report) = session.final_report {
        Ok(Json(report.clone()))
    } else {
        let report = quiz_engine::generate_diagnosis_report(&state.data_dir, &session).await?;
        Ok(Json(report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_listener_addr_uses_loopback_only() {
        assert_eq!(listener_addr(14200), "127.0.0.1:14200");
    }

    #[test]
    fn missing_diagnosis_session_sends_error_event() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        send_missing_session_error(&tx, "missing-session");

        match rx.try_recv().expect("an error event should be sent") {
            quiz_engine::DiagnosisStreamEvent::Error { message } => {
                assert!(message.contains("missing-session"));
            }
            other => panic!("expected error event, got {:?}", event_name(&other)),
        }
    }

    fn event_name(event: &quiz_engine::DiagnosisStreamEvent) -> &'static str {
        match event {
            quiz_engine::DiagnosisStreamEvent::Initial { .. } => "initial",
            quiz_engine::DiagnosisStreamEvent::FollowUp { .. } => "follow_up",
            quiz_engine::DiagnosisStreamEvent::Report { .. } => "report",
            quiz_engine::DiagnosisStreamEvent::Error { .. } => "error",
        }
    }
}
