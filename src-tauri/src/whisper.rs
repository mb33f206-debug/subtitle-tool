use std::fs;
use std::path::PathBuf;

use crate::bin_path::{ensure_ascii_path, find_binary, find_model, run_command, safe_temp_dir};
use crate::progress::{emit_log, emit_progress};

/// Run whisper.cpp speech recognition
pub fn run_whisper(
    app: &tauri::AppHandle,
    audio_path: &str,
    language: &str,
) -> Result<String, String> {
    emit_log(app, "info", "正在執行語音辨識...");
    emit_progress(app, "whisper", 20, "正在執行語音辨識...");

    let model_raw = find_model().ok_or_else(|| {
        "找不到 whisper 模型檔 (ggml-small.bin)。請將模型放在 models/ 資料夾中。".to_string()
    })?;

    let whisper = find_binary("whisper-cpp");

    // whisper.cpp on Windows can't handle Unicode paths — ensure ASCII
    let model = ensure_ascii_path(&model_raw, "whisper_model")?;
    let audio = ensure_ascii_path(audio_path, "whisper_audio")?;

    emit_log(app, "info", &format!("使用 whisper: {}", whisper));
    emit_log(app, "info", &format!("使用模型: {}", model));

    // whisper.cpp outputs SRT to {output_base}.srt — must be ASCII-safe path
    let audio_p = PathBuf::from(&audio);
    let output_base = safe_temp_dir().join(
        audio_p
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
            + "_whisper",
    );
    let output_base_str = output_base.to_string_lossy().to_string();

    let mut args = vec!["-m", &model, "-f", &audio, "-osrt", "-of", &output_base_str];
    if language != "auto" {
        args.extend_from_slice(&["-l", language]);
    }

    run_command(
        &whisper,
        &args,
        "請確認 bin/ 資料夾中有 whisper-cpp。",
    )?;

    let srt_path = format!("{}.srt", output_base_str);
    let srt_content = fs::read_to_string(&srt_path)
        .map_err(|e| format!("無法讀取 whisper 輸出: {}", e))?;

    // Clean up temp SRT
    let _ = fs::remove_file(&srt_path);

    emit_log(app, "info", "語音辨識完成 (whisper.cpp)");
    emit_progress(app, "whisper", 60, "語音辨識完成");

    Ok(srt_content)
}
