use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::audio::extract_audio;
use crate::bin_path::cleanup_ascii_temps;
use crate::gemini::call_gemini;
use crate::progress::{emit_log, emit_progress};
use crate::settings::{load_settings, save_settings_to_file, settings_path};
use crate::subtitle::{entries_to_ass, entries_to_txt, parse_srt};
use crate::whisper::run_whisper;

#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub video_path: String,
    pub language: String,
    pub use_gemini: bool,
    pub translate_to: String,
    pub output_format: String,
    #[serde(default)]
    pub output_dir: String,
}

#[tauri::command]
pub async fn process_video(app: tauri::AppHandle, req: ProcessRequest) -> Result<String, String> {
    emit_log(&app, "info", "========== 開始處理 ==========");
    emit_log(&app, "info", &format!("影片: {}", req.video_path));
    emit_log(&app, "info", &format!("語言: {}", req.language));
    emit_log(
        &app,
        "info",
        &format!("Gemini 校正: {}", if req.use_gemini { "是" } else { "否" }),
    );
    if !req.translate_to.is_empty() {
        emit_log(&app, "info", &format!("翻譯目標: {}", req.translate_to));
    }
    emit_log(&app, "info", &format!("輸出格式: {}", req.output_format));

    // Resolve output directory
    let video_path = Path::new(&req.video_path);
    let output_dir = if req.output_dir.is_empty() {
        video_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        let dir = PathBuf::from(&req.output_dir);
        fs::create_dir_all(&dir).map_err(|e| format!("無法建立輸出目錄: {}", e))?;
        dir
    };
    emit_log(
        &app,
        "info",
        &format!("輸出目錄: {}", output_dir.display()),
    );
    emit_progress(&app, "start", 0, "準備中...");

    // Step 1: Extract audio
    let audio_path = extract_audio(&app, &req.video_path)?;

    // Step 2: Run whisper
    let srt_raw = run_whisper(&app, &audio_path, &req.language)?;

    // Clean up temp audio and any ASCII-safe temp copies (model, audio hard links)
    let _ = fs::remove_file(&audio_path);
    cleanup_ascii_temps();

    let needs_gemini = req.use_gemini || !req.translate_to.is_empty();
    let mut srt_final = srt_raw;

    // Load settings and create HTTP client once for Gemini steps (only if needed)
    let gemini_ctx = if needs_gemini {
        let path = settings_path(&app)?;
        let s = load_settings(&path)?;
        if s.gemini.api_key.is_empty() || s.gemini.base_url.is_empty() {
            return Err("Gemini API 設定不完整，請先在設定中配置 API Key 和 Base URL".to_string());
        }
        let c = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("建立 HTTP 客戶端失敗: {}", e))?;
        Some((s.gemini, c))
    } else {
        None
    };

    // Step 3: Gemini correction
    if req.use_gemini {
        let (ref settings, ref client) = *gemini_ctx.as_ref().unwrap();
        emit_log(&app, "info", "正在使用 Gemini AI 校正字幕...");
        emit_progress(&app, "gemini", 65, "Gemini AI 校正中...");

        let fix_prompt = "你是一個專業的字幕校對員。我會給你一段影片和語音辨識軟體自動產生的 SRT 字幕。請你觀看影片，對照字幕，修正錯字、標點、斷句。保持 SRT 格式不變（序號、時間軸完全不動），只修正文字內容，直接輸出修正後的完整 SRT。";

        match call_gemini(&app, client, settings, fix_prompt, &srt_final).await {
            Ok(corrected) => {
                srt_final = corrected;
                emit_log(&app, "success", "Gemini 校正完成");
            }
            Err(e) => {
                emit_log(
                    &app,
                    "error",
                    &format!("Gemini 校正失敗，使用原始結果: {}", e),
                );
            }
        }
        emit_progress(&app, "gemini", 80, "校正完成");
    }

    // Step 4: Translation
    if !req.translate_to.is_empty() {
        let (ref settings, ref client) = *gemini_ctx.as_ref().unwrap();
        emit_log(
            &app,
            "info",
            &format!("正在翻譯為 {}...", req.translate_to),
        );
        emit_progress(&app, "translate", 82, "翻譯中...");

        let target_lang = match req.translate_to.as_str() {
            "en" => "英文",
            "ja" => "日文",
            "ko" => "韓文",
            "zh-CN" => "簡體中文",
            _ => &req.translate_to,
        };
        let translate_prompt = format!(
            "你是一個專業的字幕翻譯員。將字幕翻譯成{}，保持 SRT 格式不變，直接輸出翻譯後的完整 SRT。",
            target_lang
        );

        match call_gemini(&app, client, settings, &translate_prompt, &srt_final).await {
            Ok(translated) => {
                srt_final = translated;
                emit_log(&app, "success", "翻譯完成");
            }
            Err(e) => {
                emit_log(&app, "error", &format!("翻譯失敗: {}", e));
            }
        }
        emit_progress(&app, "translate", 90, "翻譯完成");
    }

    // Step 5: Write output files
    emit_progress(&app, "output", 92, "正在輸出檔案...");

    let stem = video_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    let mut output_files = Vec::new();

    let write_srt = req.output_format == "srt" || req.output_format == "all";
    let write_ass = req.output_format == "ass" || req.output_format == "all";
    let write_txt = req.output_format == "txt" || req.output_format == "all";

    // Parse SRT once for ASS/TXT conversion
    let entries = if write_ass || write_txt {
        Some(parse_srt(&srt_final))
    } else {
        None
    };

    let write_output = |path: &Path, content: &str, ext: &str| -> Result<String, String> {
        fs::write(path, content).map_err(|e| format!("寫入 {} 失敗: {}", ext, e))?;
        Ok(path.to_string_lossy().to_string())
    };

    if write_srt {
        let p = output_dir.join(format!("{}.srt", stem));
        output_files.push(write_output(&p, &srt_final, "SRT")?);
        emit_log(&app, "success", &format!("已輸出: {}", p.display()));
    }

    if write_ass {
        let ass = entries_to_ass(entries.as_ref().unwrap());
        let p = output_dir.join(format!("{}.ass", stem));
        output_files.push(write_output(&p, &ass, "ASS")?);
        emit_log(&app, "success", &format!("已輸出: {}", p.display()));
    }

    if write_txt {
        let txt = entries_to_txt(entries.as_ref().unwrap());
        let p = output_dir.join(format!("{}.txt", stem));
        output_files.push(write_output(&p, &txt, "TXT")?);
        emit_log(&app, "success", &format!("已輸出: {}", p.display()));
    }

    emit_progress(&app, "done", 100, "處理完成！");
    emit_log(&app, "success", "========== 處理完成 ==========");

    Ok(output_files.join("\n"))
}

#[tauri::command]
pub async fn fetch_models(base_url: String, api_key: String) -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("建立 HTTP 客戶端失敗: {}", e))?;
    let url = format!("{}/models", base_url.trim_end_matches('/'));

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("無法連線: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API 錯誤 ({})", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析失敗: {}", e))?;

    let mut models: Vec<String> = json["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    models.sort();
    Ok(models)
}

#[tauri::command]
pub async fn get_settings(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let path = settings_path(&app)?;
    let settings = load_settings(&path)?;
    serde_json::to_value(&settings).map_err(|e| format!("序列化設定失敗: {}", e))
}

#[tauri::command]
pub async fn save_settings(app: tauri::AppHandle, settings: serde_json::Value) -> Result<(), String> {
    let path = settings_path(&app)?;
    save_settings_to_file(&path, &settings)
}

#[tauri::command]
pub async fn save_log_file(path: String, content: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("無法建立目錄: {}", e))?;
    }
    fs::write(p, &content).map_err(|e| format!("寫入日誌失敗: {}", e))
}
