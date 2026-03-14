use serde::Serialize;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize)]
pub struct LogEvent {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub stage: String,
    pub percent: u32,
    pub message: String,
}

pub fn emit_progress(app: &tauri::AppHandle, stage: &str, percent: u32, message: &str) {
    let _ = app.emit(
        "progress",
        ProgressEvent {
            stage: stage.to_string(),
            percent,
            message: message.to_string(),
        },
    );
}

pub fn emit_log(app: &tauri::AppHandle, level: &str, message: &str) {
    let _ = app.emit(
        "log",
        LogEvent {
            level: level.to_string(),
            message: message.to_string(),
        },
    );
}
