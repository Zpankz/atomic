//! Import commands for importing notes from external applications

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Result of an import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: i32,
    pub skipped: i32,
    pub errors: i32,
    pub tags_created: i32,
    pub tags_linked: i32,
}

/// Import notes from an Obsidian vault
///
/// This command spawns the Node.js import script and streams progress back to the frontend.
#[tauri::command]
pub async fn import_obsidian_vault(
    app: AppHandle,
    vault_path: String,
    max_notes: Option<i32>,
) -> Result<ImportResult, String> {
    // Get the path to the import script
    // In development, it's relative to the project root
    // In production, we need to bundle it or use a different approach

    let mut cmd = Command::new("node");

    // Build the command arguments
    let script_path = get_script_path(&app, "import/obsidian.js")?;

    cmd.arg(&script_path);
    cmd.arg(&vault_path);
    cmd.arg("--json-output");

    if let Some(max) = max_notes {
        cmd.arg("--max");
        cmd.arg(max.to_string());
    }

    // Set up stdio
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Spawn the process
    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn import process: {}", e))?;

    // Read stdout for the JSON result
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    // Read all output
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut json_output = String::new();
    let mut error_output = String::new();

    // Read stdout lines (looking for JSON output)
    while let Ok(Some(line)) = stdout_reader.next_line().await {
        // The script outputs JSON as the last line when using --json-output
        if line.starts_with('{') {
            json_output = line;
        }
    }

    // Read any stderr
    while let Ok(Some(line)) = stderr_reader.next_line().await {
        if !error_output.is_empty() {
            error_output.push('\n');
        }
        error_output.push_str(&line);
    }

    // Wait for the process to finish
    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for import process: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Import process failed with exit code {:?}: {}",
            status.code(),
            error_output
        ));
    }

    // Parse the JSON result
    if json_output.is_empty() {
        return Err("No output from import script".to_string());
    }

    serde_json::from_str(&json_output).map_err(|e| format!("Failed to parse import result: {}", e))
}

/// Get the path to a script file
fn get_script_path(app: &AppHandle, script_name: &str) -> Result<String, String> {
    // In development, scripts are in the project root
    // Try to find the scripts directory relative to the app

    // First, try the resource directory (for bundled apps)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let script_path = resource_dir.join("scripts").join(script_name);
        if script_path.exists() {
            return Ok(script_path.to_string_lossy().to_string());
        }
    }

    // For development, try relative to current directory
    let dev_path = std::path::Path::new("scripts").join(script_name);
    if dev_path.exists() {
        return Ok(dev_path.to_string_lossy().to_string());
    }

    // Try from the app's installation directory
    // On macOS, the app bundle is at /Applications/Atomic.app/Contents/MacOS/Atomic
    // Scripts would be at /Applications/Atomic.app/Contents/Resources/scripts/
    #[cfg(target_os = "macos")]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let resources_path = parent.parent().map(|p| p.join("Resources").join("scripts").join(script_name));
                if let Some(path) = resources_path {
                    if path.exists() {
                        return Ok(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Err(format!("Could not find script: {}", script_name))
}
