//! Settings and model discovery commands — delegates to AtomicCore

use crate::event_bridge::embedding_event_callback;
use atomic_core::AtomicCore;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_settings(core: State<AtomicCore>) -> Result<HashMap<String, String>, String> {
    core.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(
    app_handle: AppHandle,
    core: State<'_, AtomicCore>,
    key: String,
    value: String,
) -> Result<(), String> {
    let (dimension_changed, pending_count) = core
        .set_setting_with_reembed(&key, &value, embedding_event_callback(app_handle.clone()))
        .map_err(|e| e.to_string())?;

    if dimension_changed {
        eprintln!(
            "Dimension changed - {} atoms marked as pending, emitting event",
            pending_count
        );
        let _ = app_handle.emit(
            "embeddings-reset",
            serde_json::json!({
                "pending_count": pending_count,
                "reason": format!("{} changed", key)
            }),
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn test_openrouter_connection(api_key: String) -> Result<bool, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "anthropic/claude-haiku-4.5",
            "messages": [{"role": "user", "content": "Hi"}],
            "max_tokens": 5
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("API error ({}): {}", status, body))
    }
}

#[tauri::command]
pub async fn get_available_llm_models(
    core: State<'_, AtomicCore>,
) -> Result<Vec<atomic_core::providers::AvailableModel>, String> {
    use atomic_core::providers::models::{
        fetch_and_return_capabilities, get_cached_capabilities_sync, save_capabilities_cache,
    };

    let db = core.database();

    // Check cache first
    let (cached, is_stale) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        match get_cached_capabilities_sync(&conn) {
            Ok(Some(cache)) => (Some(cache.clone()), cache.is_stale()),
            Ok(None) => (None, true),
            Err(_) => (None, true),
        }
    };

    if let Some(ref cache) = cached {
        if !is_stale {
            return Ok(cache.get_models_with_structured_outputs());
        }
    }

    let client = reqwest::Client::new();
    match fetch_and_return_capabilities(&client).await {
        Ok(fresh_cache) => {
            if let Ok(conn) = db.new_connection() {
                let _ = save_capabilities_cache(&conn, &fresh_cache);
            }
            Ok(fresh_cache.get_models_with_structured_outputs())
        }
        Err(e) => {
            if let Some(cache) = cached {
                Ok(cache.get_models_with_structured_outputs())
            } else {
                Err(format!("Failed to fetch models: {}", e))
            }
        }
    }
}
