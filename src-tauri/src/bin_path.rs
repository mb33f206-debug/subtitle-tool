use std::path::PathBuf;
use std::sync::OnceLock;

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
        let in_bin = dir.join("bin").join(&bin_name);
        if in_bin.exists() {
            return in_bin.to_string_lossy().to_string();
        }

        let beside = dir.join(&bin_name);
        if beside.exists() {
            return beside.to_string_lossy().to_string();
        }
    }

    name.to_string()
}

/// Find the whisper model file (ggml-small.bin).
/// Search order: 1. models/ next to exe  2. next to exe  3. home cache  4. cwd
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

/// Sanitize a string to contain only safe ASCII filename characters.
/// Replaces non-ASCII and filesystem-unsafe chars with underscore.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Cached ASCII-safe temp directory. Computed once per process.
static SAFE_TEMP: OnceLock<PathBuf> = OnceLock::new();

/// Get a guaranteed ASCII-safe temp directory.
/// On Windows, if %TEMP% contains non-ASCII (e.g. Chinese username),
/// falls back to %LOCALAPPDATA%\Temp, then C:\Temp.
pub fn safe_temp_dir() -> PathBuf {
    SAFE_TEMP.get_or_init(|| {
        let tmp = std::env::temp_dir();
        if tmp.to_string_lossy().is_ascii() {
            return tmp;
        }
        // Try %LOCALAPPDATA%\Temp first (always exists, user-writable)
        if cfg!(target_os = "windows") {
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                let p = PathBuf::from(&local).join("Temp");
                if p.to_string_lossy().is_ascii() {
                    let _ = std::fs::create_dir_all(&p);
                    return p;
                }
            }
        }
        let fallback = if cfg!(target_os = "windows") {
            PathBuf::from("C:\\Temp")
        } else {
            PathBuf::from("/tmp")
        };
        let _ = std::fs::create_dir_all(&fallback);
        fallback
    }).clone()
}

/// Ensure a file path is safe for external tools that can't handle Unicode paths (e.g. whisper.cpp on Windows).
/// If the path contains non-ASCII characters, creates a hard link (or copy) in an ASCII-safe temp directory.
pub fn ensure_ascii_path(original: &str, prefix: &str) -> Result<String, String> {
    if original.is_ascii() {
        return Ok(original.to_string());
    }

    let src = std::path::Path::new(original);
    let filename = src.file_name().unwrap_or_default().to_string_lossy();
    let safe_path = safe_temp_dir().join(format!("{}_{}", prefix, sanitize_filename(&filename)));
    let safe_str = safe_path.to_string_lossy().to_string();

    // Reuse existing copy if same size AND same modification time
    if safe_path.exists() {
        if let (Ok(src_meta), Ok(dst_meta)) = (src.metadata(), safe_path.metadata()) {
            let same_size = src_meta.len() == dst_meta.len();
            let same_mtime = src_meta.modified().ok() == dst_meta.modified().ok();
            if same_size && same_mtime {
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

/// Run an external command and return its output.
/// If the command path contains non-ASCII, copies it to an ASCII-safe location first.
/// Sets working directory to the binary's parent so Windows can find sibling DLLs.
pub fn run_command(cmd: &str, args: &[&str], error_hint: &str) -> Result<std::process::Output, String> {
    let safe_cmd = if !cmd.is_ascii() {
        ensure_ascii_path(cmd, "cmd")?
    } else {
        cmd.to_string()
    };

    let mut command = std::process::Command::new(&safe_cmd);
    command.args(args);

    // Set working directory to binary's parent for DLL discovery on Windows
    if let Some(parent) = std::path::Path::new(&safe_cmd).parent() {
        if parent.exists() && parent.to_string_lossy().len() > 0 {
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
