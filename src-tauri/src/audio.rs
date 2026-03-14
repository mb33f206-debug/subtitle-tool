use std::path::Path;

use crate::bin_path::{find_binary, run_command, safe_temp_dir};
use crate::progress::{emit_log, emit_progress};

/// Extract audio from video using bundled or system ffmpeg
pub fn extract_audio(app: &tauri::AppHandle, video_path: &str) -> Result<String, String> {
    emit_log(app, "info", &format!("正在從影片提取音訊: {}", video_path));
    emit_progress(app, "extract", 5, "正在提取音訊...");

    let ffmpeg = find_binary("ffmpeg");
    emit_log(app, "info", &format!("使用 ffmpeg: {}", ffmpeg));

    let video = Path::new(video_path);
    let file_stem = video
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    // Sanitize filename to ASCII for whisper-cpp compatibility downstream
    let safe_stem: String = file_stem.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    let audio_path = safe_temp_dir().join(format!("{}_subtitle_temp.wav", safe_stem));
    let audio_str = audio_path.to_string_lossy().to_string();

    run_command(
        &ffmpeg,
        &["-y", "-i", video_path, "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le", &audio_str],
        "請確認 bin/ 資料夾中有 ffmpeg。",
    )?;

    emit_log(app, "info", "音訊提取完成");
    emit_progress(app, "extract", 15, "音訊提取完成");
    Ok(audio_str)
}
