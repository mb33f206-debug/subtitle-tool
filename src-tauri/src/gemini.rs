use crate::progress::emit_log;
use crate::settings::GeminiSettings;

const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY_SECS: u64 = 10;

/// Call Gemini API for text correction, with automatic retry on 429 (rate limit).
pub async fn call_gemini(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    settings: &GeminiSettings,
    prompt: &str,
    srt: &str,
) -> Result<String, String> {
    let url = format!(
        "{}/chat/completions",
        settings.base_url.trim_end_matches('/')
    );

    let body = serde_json::json!({
        "model": settings.model,
        "messages": [
            { "role": "system", "content": prompt },
            { "role": "user", "content": srt }
        ],
        "temperature": 0.3
    });

    let mut last_error = String::new();

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay = INITIAL_RETRY_DELAY_SECS * (1 << (attempt - 1)); // 10s, 20s, 40s
            emit_log(
                app,
                "warn",
                &format!("API 限流 (429)，等待 {}s 後重試 ({}/{})...", delay, attempt, MAX_RETRIES),
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        }

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", settings.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Gemini API 請求失敗: {}", e))?;

        let status = resp.status();

        // Retry on 429 Too Many Requests
        if status.as_u16() == 429 {
            let text = resp.text().await.unwrap_or_default();
            last_error = format!("Gemini API 錯誤 (429 Too Many Requests): {}", text);
            if attempt < MAX_RETRIES {
                continue;
            }
            return Err(format!("{}（已重試 {} 次）", last_error, MAX_RETRIES));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| format!("讀取回應失敗: {}", e))?;

        if !status.is_success() {
            return Err(format!("Gemini API 錯誤 ({}): {}", status, text));
        }

        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("解析回應 JSON 失敗: {}", e))?;

        return json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| {
                let s = s.trim();
                if s.starts_with("```srt") || s.starts_with("```") {
                    let s = s.trim_start_matches("```srt").trim_start_matches("```");
                    if let Some(end) = s.rfind("```") {
                        return s[..end].trim().to_string();
                    }
                    return s.trim().to_string();
                }
                s.to_string()
            })
            .ok_or_else(|| "Gemini API 回應格式異常".to_string());
    }

    Err(last_error)
}
