use axum::{
    extract::{Json, Path as AxumPath, Query, State},
    http::{header, HeaderValue, Method},
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::models::diagnosis::{DiagnosisReport, DiagnosisSession};
use crate::models::mistake::{MistakeEntry, MistakeFilter};
use crate::models::note::{NoteContent, NoteTreeNode};
use crate::models::quiz::QuizStreamParams;
use crate::models::settings::Settings;
use crate::services::llm_service::ConnectionTestResult;
use crate::services::{
    config, diagnosis_session_service, fs_service, llm_service, mistake_service, note_service,
    quiz_engine, storage,
};

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

fn allowed_cors_origins() -> Vec<HeaderValue> {
    vec![
        HeaderValue::from_static("http://localhost:1420"),
        HeaderValue::from_static("http://127.0.0.1:1420"),
    ]
}

fn local_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(allowed_cors_origins())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE])
}

fn send_missing_session_error(
    tx: &UnboundedSender<quiz_engine::DiagnosisStreamEvent>,
    session_id: &str,
) {
    let _ = tx.send(quiz_engine::DiagnosisStreamEvent::Error {
        message: format!("Session {} not found", session_id),
    });
}

fn load_diagnosis_session(state: &AppState, session_id: &str) -> Result<DiagnosisSession, String> {
    if let Ok(mut sessions) = state.diagnosis_sessions.lock() {
        if let Some(session) = sessions.remove(session_id) {
            return Ok(session);
        }
    }
    diagnosis_session_service::load_session(&state.data_dir, session_id)
}

fn cache_diagnosis_session(state: &AppState, session: DiagnosisSession) -> Result<(), String> {
    diagnosis_session_service::save_session(&state.data_dir, &session)?;
    let mut sessions = state
        .diagnosis_sessions
        .lock()
        .map_err(|_| "Failed to lock diagnosis sessions".to_string())?;
    sessions.insert(session.session_id.clone(), session);
    Ok(())
}

fn finish_diagnosis_session(state: &AppState, session: DiagnosisSession) -> Result<(), String> {
    if session.final_report.is_some() {
        diagnosis_session_service::delete_session(&state.data_dir, &session.session_id)?;
        let mut sessions = state
            .diagnosis_sessions
            .lock()
            .map_err(|_| "Failed to lock diagnosis sessions".to_string())?;
        sessions.remove(&session.session_id);
        Ok(())
    } else {
        cache_diagnosis_session(state, session)
    }
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
    let serve_dir = ServeDir::new(&dist_dir).fallback(ServeFile::new(index_path));

    let app = Router::new()
        .route("/api/notes/scan", get(scan_notes_handler))
        .route("/api/notes/read", get(read_note_handler))
        .route("/api/notes/asset", get(read_note_asset_handler))
        .route(
            "/api/settings",
            get(get_settings_handler).post(save_settings_handler),
        )
        .route("/api/test-connection", post(test_connection_handler))
        .route("/api/data/backup", post(backup_data_handler))
        .route("/api/prompt-templates", get(list_prompt_templates_handler))
        .route(
            "/api/mistakes",
            get(load_mistakes_handler).post(save_mistake_handler),
        )
        .route("/api/quiz/generate", post(generate_quiz_handler))
        .route("/api/quiz/diagnose", post(submit_diagnosis_handler))
        .route(
            "/api/quiz/diagnose/{session_id}/follow_up",
            post(diagnose_follow_up_handler),
        )
        .route(
            "/api/quiz/diagnose/{session_id}/report",
            get(generate_report_handler),
        )
        .fallback_service(serve_dir)
        .layer(local_cors_layer())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(listener_addr(port))
        .await
        .expect("Failed to bind TCP listener");

    println!("KnowteQuiz web server running on http://localhost:{}", port);

    axum::serve(listener, app).await.expect("Server error");
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

#[derive(Deserialize)]
struct ReadNoteAssetQuery {
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

async fn read_note_asset_handler(
    Query(query): Query<ReadNoteAssetQuery>,
) -> Result<Response, String> {
    let path = PathBuf::from(&query.path);
    let content_type = asset_content_type(&path)
        .ok_or_else(|| format!("Unsupported asset type: {}", query.path))?;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|err| format!("Failed to read asset {}: {}", query.path, err))?;

    Ok(([(header::CONTENT_TYPE, content_type)], bytes).into_response())
}

fn asset_content_type(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => Some("image/png"),
        Some("jpg") | Some("jpeg") => Some("image/jpeg"),
        Some("gif") => Some("image/gif"),
        Some("webp") => Some("image/webp"),
        Some("bmp") => Some("image/bmp"),
        Some("svg") => Some("image/svg+xml"),
        _ => None,
    }
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
) -> Result<Json<ConnectionTestResult>, String> {
    let settings = config::get_settings_path(&state.data_dir)?;
    Ok(Json(llm_service::test_connection(&settings.llm).await))
}

async fn backup_data_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<storage::DataBackupResult>, String> {
    Ok(Json(storage::backup_data_files_path(&state.data_dir)?))
}

async fn list_prompt_templates_handler() -> Result<Json<Vec<(String, String, String)>>, String> {
    Ok(Json(crate::utils::prompt_templates::list_template_sets()))
}

async fn load_mistakes_handler(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<MistakeFilter>,
) -> Result<Json<Vec<MistakeEntry>>, String> {
    let mistakes = mistake_service::load_mistakes(&state.data_dir, &filter)?;
    Ok(Json(mistakes))
}

async fn save_mistake_handler(
    State(state): State<Arc<AppState>>,
    Json(entry): Json<MistakeEntry>,
) -> Result<Json<bool>, String> {
    mistake_service::save_mistake(&state.data_dir, entry)?;
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
    let correct_answer = payload["correct_answer"].as_str().unwrap_or("").to_string();
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
            if let Err(err) = cache_diagnosis_session(&app_state, session) {
                let _ = tx.send(quiz_engine::DiagnosisStreamEvent::Error { message: err });
            }
        }
    }

    tokio::spawn(async move {
        match quiz_engine::submit_diagnosis_initial(
            &app_state.data_dir,
            &question,
            &correct_answer,
            &user_answer,
            &user_reasoning,
            &note_path,
            tx.clone(),
        )
        .await
        {
            Ok(initial_round) => {
                let updated_session =
                    if let Ok(mut sessions_lock) = app_state.diagnosis_sessions.lock() {
                        sessions_lock.get_mut(&session_id).map(|session| {
                            session.conversation.push(initial_round);
                            session.clone()
                        })
                    } else {
                        None
                    };

                if let Some(session) = updated_session {
                    let _ = diagnosis_session_service::save_session(&app_state.data_dir, &session);
                }
            }
            Err(err) => {
                let _ = tx.send(quiz_engine::DiagnosisStreamEvent::Error { message: err });
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
        let mut session = match load_diagnosis_session(&app_state, &session_id) {
            Ok(session) => session,
            Err(_) => {
                send_missing_session_error(&tx, &session_id);
                return;
            }
        };

        if let Err(err) = quiz_engine::diagnose_follow_up(
            &app_state.data_dir,
            &mut session,
            &user_reply,
            tx.clone(),
        )
        .await
        {
            let _ = tx.send(quiz_engine::DiagnosisStreamEvent::Error { message: err });
            let _ = cache_diagnosis_session(&app_state, session);
            return;
        }

        if let Err(err) = finish_diagnosis_session(&app_state, session) {
            let _ = tx.send(quiz_engine::DiagnosisStreamEvent::Error { message: err });
        }
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
    let mut session = load_diagnosis_session(&state, &session_id)
        .map_err(|_| format!("Session {} not found", session_id))?;

    if let Some(ref report) = session.final_report {
        finish_diagnosis_session(&state, session.clone())?;
        Ok(Json(report.clone()))
    } else {
        let report = quiz_engine::generate_diagnosis_report(&state.data_dir, &session).await?;
        session.final_report = Some(report.clone());
        finish_diagnosis_session(&state, session)?;
        Ok(Json(report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[test]
    fn default_listener_addr_uses_loopback_only() {
        assert_eq!(listener_addr(14200), "127.0.0.1:14200");
    }

    #[test]
    fn cors_allows_only_local_dev_origins() {
        let origins = allowed_cors_origins();

        assert_eq!(origins.len(), 2);
        assert!(origins.contains(&HeaderValue::from_static("http://localhost:1420")));
        assert!(origins.contains(&HeaderValue::from_static("http://127.0.0.1:1420")));
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

    #[test]
    fn load_diagnosis_session_falls_back_to_persisted_session() {
        let data_dir = temp_data_dir("load_diagnosis_session_falls_back_to_persisted_session");
        let session = diagnosis_session("session-from-disk");
        diagnosis_session_service::save_session(&data_dir, &session)
            .expect("session should be persisted");
        let state = AppState {
            data_dir,
            diagnosis_sessions: Mutex::new(HashMap::new()),
        };

        let loaded = load_diagnosis_session(&state, "session-from-disk")
            .expect("session should load from disk");

        assert_eq!(loaded.session_id, "session-from-disk");
        assert_eq!(loaded.question, "What is ownership?");
    }

    #[test]
    fn asset_content_type_allows_common_image_formats_only() {
        assert_eq!(
            asset_content_type(Path::new("diagram.png")),
            Some("image/png")
        );
        assert_eq!(
            asset_content_type(Path::new("photo.JPEG")),
            Some("image/jpeg")
        );
        assert_eq!(
            asset_content_type(Path::new("clip.webp")),
            Some("image/webp")
        );
        assert_eq!(asset_content_type(Path::new("note.md")), None);
    }

    #[tokio::test]
    async fn submit_diagnosis_stream_reports_initial_failures() {
        let data_dir = temp_data_dir("submit_diagnosis_stream_reports_initial_failures");
        let state = Arc::new(AppState {
            data_dir,
            diagnosis_sessions: Mutex::new(HashMap::new()),
        });
        let app = Router::new()
            .route("/diagnose", post(submit_diagnosis_handler))
            .with_state(state);
        let body = serde_json::json!({
            "session_id": "session-1",
            "question": "Which claim is true?",
            "correct_answer": "B",
            "user_answer": "A",
            "user_reasoning": "I guessed.",
            "note_path": "D:/missing-note.md"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/diagnose")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("request should complete");

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_text = String::from_utf8(bytes.to_vec()).expect("SSE body should be utf8");

        assert!(body_text.contains("\"event\":\"error\""));
        assert!(body_text.contains("File does not exist"));
    }

    fn event_name(event: &quiz_engine::DiagnosisStreamEvent) -> &'static str {
        match event {
            quiz_engine::DiagnosisStreamEvent::Initial { .. } => "initial",
            quiz_engine::DiagnosisStreamEvent::FollowUp { .. } => "follow_up",
            quiz_engine::DiagnosisStreamEvent::Report { .. } => "report",
            quiz_engine::DiagnosisStreamEvent::Error { .. } => "error",
        }
    }

    fn temp_data_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-web-server-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    fn diagnosis_session(id: &str) -> DiagnosisSession {
        DiagnosisSession {
            session_id: id.to_string(),
            question: "What is ownership?".to_string(),
            user_answer: "A".to_string(),
            user_reasoning: "Because moves copy.".to_string(),
            note_path: "D:/notes/rust.md".to_string(),
            note_content: "Ownership note".to_string(),
            conversation: vec![],
            current_round: 0,
            max_rounds: 3,
            final_report: None,
        }
    }
}
