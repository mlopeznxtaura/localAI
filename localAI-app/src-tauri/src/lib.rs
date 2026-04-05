use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use serde::{Deserialize, Serialize};

fn pipeline_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("intent-to-software")
}

fn output_dir() -> PathBuf {
    pipeline_dir().join("output")
}

const GITHUB_OAUTH_CLIENT_ID: &str = "Ov23liGAjKwETmJFZqAk";

fn python_exe() -> Result<OsString, String> {
    // Prefer the pipeline-local venv if present (works around PEP 668 on Ubuntu).
    let pipeline = pipeline_dir();

    let venv_linux = pipeline.join(".venv").join("bin").join("python");
    if venv_linux.exists() {
        return Ok(venv_linux.into_os_string());
    }

    let venv_windows = pipeline.join(".venv").join("Scripts").join("python.exe");
    if venv_windows.exists() {
        return Ok(venv_windows.into_os_string());
    }

    // Otherwise, prefer python3 when available (WSL/mac/linux), else fall back to python.
    match Command::new("python3").arg("--version").output() {
        Ok(_) => Ok(OsString::from("python3")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(OsString::from("python")),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubTokenFile {
    access_token: String,
}

fn token_file_path() -> PathBuf {
    // Repo-local so it works in dev/WSL without extra OS integration.
    // This path is gitignored (see repo `.gitignore`).
    pipeline_dir().join(".localai").join("github_token.json")
}

fn slugify_repo_name(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in input.chars().flat_map(|c| c.to_lowercase()) {
        let is_alnum = ch.is_ascii_alphanumeric();
        if is_alnum {
            out.push(ch);
            prev_dash = false;
            continue;
        }

        if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }

    if out.len() > 60 {
        out.truncate(60);
        while out.ends_with('-') {
            out.pop();
        }
    }

    if out.is_empty() {
        "localai-output".to_string()
    } else {
        out
    }
}

#[tauri::command]
fn suggest_repo_name() -> String {
    let pipeline = pipeline_dir();
    let intent_path = pipeline.join("compressed_intent.txt");
    let prompt_path = pipeline.join("user_prompt.txt");

    let text = fs::read_to_string(intent_path)
        .or_else(|_| fs::read_to_string(prompt_path))
        .unwrap_or_else(|_| "localai-output".to_string());

    slugify_repo_name(text.trim())
}

fn read_saved_token() -> Option<String> {
    let p = token_file_path();
    let raw = fs::read_to_string(p).ok()?;
    let obj: GitHubTokenFile = serde_json::from_str(&raw).ok()?;
    Some(obj.access_token)
}

fn save_token(token: &str) -> Result<(), String> {
    let p = token_file_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let obj = GitHubTokenFile {
        access_token: token.to_string(),
    };
    let raw = serde_json::to_string(&obj).map_err(|e| e.to_string())?;
    fs::write(p, raw).map_err(|e| e.to_string())
}

fn delete_token() -> Result<(), String> {
    let p = token_file_path();
    if p.exists() {
        fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn ensure_gh_logged_in(token: &str) -> Result<(), String> {
    // If gh is already authed, do nothing.
    let status = Command::new("gh")
        .arg("auth")
        .arg("status")
        .output()
        .map_err(|e| e.to_string())?;
    if status.status.success() {
        return Ok(());
    }

    let mut child = Command::new("gh")
        .arg("auth")
        .arg("login")
        .arg("--with-token")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    {
        use std::io::Write;
        let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        stdin.write_all(token.as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
    }

    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

#[tauri::command]
fn run_step(step: u32, prompt: String) -> Result<(), String> {
    let working_dir = pipeline_dir();

    if step == 1 {
        fs::write(working_dir.join("user_prompt.txt"), prompt).map_err(|e| e.to_string())?;
    }

    let script = match step {
        1 => "step1_compress.py",
        2 => "step2_mockui.py",
        3 => "step3_parse.py",
        4 => "step4_dag.py",
        5 => "step5_tasks.py",
        6 => "step6_build.py",
        _ => return Err("Invalid step".to_string()),
    };

    let mut cmd = Command::new(python_exe()?);
    cmd.arg(script).current_dir(&working_dir);

    let output = cmd.output().map_err(|e| e.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let msg = format!(
        "Step {step} failed ({script}).\n\n--- stdout ---\n{stdout}\n\n--- stderr ---\n{stderr}"
    );

    let _ = fs::write(pipeline_dir().join("ui_last_error.txt"), &msg);
    Err(msg)
}

#[tauri::command]
fn get_output_path() -> String {
    output_dir()
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| output_dir().to_string_lossy().to_string())
}

#[tauri::command]
fn open_output_folder(path: String) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(&path).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(&path).spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(&path).spawn();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubStatus {
    authenticated: bool,
}

#[tauri::command]
fn github_status() -> GitHubStatus {
    let authed = Command::new("gh")
        .arg("auth")
        .arg("status")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Treat a saved token as authenticated even if gh isn’t available.
    GitHubStatus {
        authenticated: authed || read_saved_token().is_some(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceStartResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[tauri::command]
fn github_device_start(scopes: Vec<String>) -> Result<DeviceStartResponse, String> {
    let client_id = std::env::var("LOCALAI_GITHUB_CLIENT_ID").unwrap_or_else(|_| GITHUB_OAUTH_CLIENT_ID.to_string());

    let scope = if scopes.is_empty() {
        "public_repo".to_string()
    } else {
        scopes.join(" ")
    };

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", client_id.as_str()), ("scope", scope.as_str())])
        .send()
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub device start failed: HTTP {}", resp.status()));
    }

    resp.json::<DeviceStartResponse>().map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct DevicePollSuccess {
    access_token: String,
    token_type: String,
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DevicePollError {
    error: String,
    error_description: Option<String>,
    error_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
enum DevicePollResult {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "success")]
    Success { access_token: String, scope: String },
    #[serde(rename = "error")]
    Error { error: String, description: String },
}

#[tauri::command]
fn github_device_poll(device_code: String) -> Result<DevicePollResult, String> {
    let client_id = std::env::var("LOCALAI_GITHUB_CLIENT_ID").unwrap_or_else(|_| GITHUB_OAUTH_CLIENT_ID.to_string());

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("device_code", device_code.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub device poll failed: HTTP {}", resp.status()));
    }

    let raw = resp.text().map_err(|e| e.to_string())?;
    if raw.contains("\"access_token\"") {
        let ok: DevicePollSuccess = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        return Ok(DevicePollResult::Success {
            access_token: ok.access_token,
            scope: ok.scope,
        });
    }

    let err: DevicePollError = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let ecode = err.error;
    if ecode == "authorization_pending" || ecode == "slow_down" {
        return Ok(DevicePollResult::Pending);
    }

    Ok(DevicePollResult::Error {
        error: ecode,
        description: err.error_description.unwrap_or_default(),
    })
}

#[tauri::command]
fn github_save_and_login(access_token: String) -> Result<(), String> {
    save_token(&access_token)?;
    // Best-effort: authenticate gh CLI too (enables export command).
    let _ = ensure_gh_logged_in(&access_token);
    Ok(())
}

#[tauri::command]
fn github_logout() -> Result<(), String> {
    delete_token()?;
    let _ = Command::new("gh").arg("auth").arg("logout").arg("-h").arg("github.com").arg("-y").output();
    Ok(())
}

fn ensure_output_gitignore(output: &PathBuf) -> Result<(), String> {
    let gitignore = output.join(".gitignore");
    if gitignore.exists() {
        return Ok(());
    }

    let contents = "\
.env\n\
.env.*\n\
node_modules/\n\
dist/\n\
build/\n\
__pycache__/\n\
*.pyc\n\
.venv/\n";

    fs::write(gitignore, contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_output_to_github(repo: String, visibility: String) -> Result<String, String> {
    let output = output_dir();
    if !output.exists() {
        return Err("Output folder does not exist yet.".to_string());
    }

    ensure_output_gitignore(&output)?;

    let gh_status = Command::new("gh")
        .arg("auth")
        .arg("status")
        .current_dir(&output)
        .output()
        .map_err(|e| e.to_string())?;
    if !gh_status.status.success() {
        // If we have a saved token, attempt to auth gh automatically.
        if let Some(token) = read_saved_token() {
            ensure_gh_logged_in(&token)?;
        } else {
            return Err("GitHub CLI not authenticated. Connect GitHub in the UI (or run: gh auth login).".to_string());
        }
    }

    if !output.join(".git").exists() {
        let init_out = Command::new("git")
            .arg("init")
            .current_dir(&output)
            .output()
            .map_err(|e| e.to_string())?;
        if !init_out.status.success() {
            return Err(String::from_utf8_lossy(&init_out.stderr).to_string());
        }
    }

    let add_out = Command::new("git")
        .arg("add")
        .arg("-A")
        .current_dir(&output)
        .output()
        .map_err(|e| e.to_string())?;
    if !add_out.status.success() {
        return Err(String::from_utf8_lossy(&add_out.stderr).to_string());
    }

    let _ = Command::new("git")
        .arg("-c")
        .arg("user.name=localAI")
        .arg("-c")
        .arg("user.email=localai@local")
        .arg("commit")
        .arg("-m")
        .arg("Generated by localAI")
        .current_dir(&output)
        .output();

    // Resolve "owner/repo" for setting default branch after creation.
    let full_repo = if repo.contains('/') {
        repo.clone()
    } else {
        let who = Command::new("gh")
            .arg("api")
            .arg("user")
            .arg("--jq")
            .arg(".login")
            .output()
            .map_err(|e| e.to_string())?;
        let owner = String::from_utf8_lossy(&who.stdout).trim().to_string();
        if owner.is_empty() {
            repo.clone()
        } else {
            format!("{owner}/{repo}")
        }
    };

    let mut create_cmd = Command::new("gh");
    create_cmd
        .arg("repo")
        .arg("create")
        .arg(&repo)
        .arg("--source")
        .arg(".")
        .arg("--push")
        .current_dir(&output);

    if visibility == "private" {
        create_cmd.arg("--private");
    } else {
        create_cmd.arg("--public");
    }

    let create_out = create_cmd.output().map_err(|e| e.to_string())?;
    if !create_out.status.success() {
        let stderr = String::from_utf8_lossy(&create_out.stderr).to_string();
        let stdout = String::from_utf8_lossy(&create_out.stdout).to_string();
        return Err(format!("GitHub export failed.\n\n--- stdout ---\n{stdout}\n\n--- stderr ---\n{stderr}"));
    }

    // Normalize branch name to main.
    let _ = Command::new("git").arg("branch").arg("-M").arg("main").current_dir(&output).output();
    let _ = Command::new("git")
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg("main")
        .current_dir(&output)
        .output();
    let _ = Command::new("gh")
        .arg("repo")
        .arg("edit")
        .arg(&full_repo)
        .arg("--default-branch")
        .arg("main")
        .current_dir(&output)
        .output();
    let _ = Command::new("git")
        .arg("push")
        .arg("origin")
        .arg("--delete")
        .arg("master")
        .current_dir(&output)
        .output();

    Ok(String::from_utf8_lossy(&create_out.stdout).to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            run_step,
            get_output_path,
            open_output_folder,
            github_status,
            github_device_start,
            github_device_poll,
            github_save_and_login,
            github_logout,
            suggest_repo_name,
            export_output_to_github
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
