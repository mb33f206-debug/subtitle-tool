use crate::settings::GeminiSettings;

/// Call Gemini API for text correction
pub async fn call_gemini(
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
            {
                "role": "system",
                "content": prompt
            },
            {
                "role": "user",
                "content": srt
            }
        ],
        "temperature": 0.3
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Gemini API 請求失敗: {}", e))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("讀取回應失敗: {}", e))?;

    if !status.is_success() {
        return Err(format!("Gemini API 錯誤 ({}): {}", status, text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("解析回應 JSON 失敗: {}", e))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| {
            // Strip markdown code blocks if present
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
        .ok_or_else(|| "Gemini API 回應格式異常".to_string())
}
