use std::path::PathBuf;

/// Get the directory containing the current executable.
fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(|p| p.to_path_buf())
}

/// Append .exe on Windows
fn binary_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

/// Find a bundled binary by searching common locations relative to the executable.
/// Search order: 1. bin/ next to exe  2. next to exe  3. system PATH (returns just the name)
pub fn find_binary(name: &str) -> String {
    let bin_name = binary_name(name);

    if let Some(dir) = exe_dir() {
        // 1. bin/ subdirectory next to executable
        let in_bin = dir.join("bin").join(&bin_name);
        if in_bin.exists() {
            return in_bin.to_string_lossy().to_string();
        }

        // 2. Directly next to executable
        let beside = dir.join(&bin_name);
        if beside.exists() {
            return beside.to_string_lossy().to_string();
        }
    }

    // 3. Fallback: rely on system PATH
    name.to_string()
}

/// Find the whisper model file (ggml-small.bin).
/// Search order: 1. models/ next to exe  2. next to exe  3. home cache  4. cwd
/// Uses lazy evaluation — stops building paths as soon as one is found.
pub fn find_model() -> Option<String> {
    let check = |p: PathBuf| -> Option<String> {
        if p.exists() { Some(p.to_string_lossy().to_string()) } else { None }
    };

    if let Some(dir) = exe_dir() {
        if let Some(s) = check(dir.join("models").join("ggml-small.bin")) { return Some(s); }
        if let Some(s) = check(dir.join("ggml-small.bin")) { return Some(s); }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    if !home.is_empty() {
        if let Some(s) = check(PathBuf::from(&home).join(".cache").join("whisper").join("ggml-small.bin")) {
            return Some(s);
        }
    }

    check(PathBuf::from("models").join("ggml-small.bin"))
}

/// Ensure a file path is safe for external tools that can't handle Unicode paths (e.g. whisper.cpp on Windows).
/// If the path contains non-ASCII characters, creates a hard link (or copy) in the temp directory.
/// Returns the safe path to use.
pub fn ensure_ascii_path(original: &str, prefix: &str) -> Result<String, String> {
    // If already ASCII, return as-is
    if original.is_ascii() {
        return Ok(original.to_string());
    }

    let src = std::path::Path::new(original);
    let filename = src.file_name().unwrap_or_default().to_string_lossy();
    let safe_path = std::env::temp_dir().join(format!("{}_{}", prefix, filename));
    let safe_str = safe_path.to_string_lossy().to_string();

    // If the safe copy already exists with same size, reuse it
    if safe_path.exists() {
        if let (Ok(src_meta), Ok(dst_meta)) = (src.metadata(), safe_path.metadata()) {
            if src_meta.len() == dst_meta.len() {
                return Ok(safe_str);
            }
        }
    }

    // Try hard link first (instant, no extra disk space), fall back to copy
    let _ = std::fs::remove_file(&safe_path);
    if std::fs::hard_link(src, &safe_path).is_ok() {
        return Ok(safe_str);
    }

    std::fs::copy(src, &safe_path)
        .map_err(|e| format!("無法複製檔案到暫存路徑: {}", e))?;

    Ok(safe_str)
}

/// Run an external command and return its stdout.
/// Returns a user-friendly error with stderr on failure.
pub fn run_command(cmd: &str, args: &[&str], error_hint: &str) -> Result<std::process::Output, String> {
    let mut command = std::process::Command::new(cmd);
    command.args(args);

    // Set working directory to the binary's parent so Windows can find sibling DLLs
    if let Some(parent) = std::path::Path::new(cmd).parent() {
        if parent.exists() {
            command.current_dir(parent);
        }
    }

    let output = command
        .output()
        .map_err(|e| format!("{} 執行失敗: {}。{}", cmd, e, error_hint))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} 錯誤: {}", cmd, stderr));
    }

    Ok(output)
}
