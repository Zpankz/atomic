//! Ollama-specific commands

use atomic_core::AtomicCore;
use tauri::State;

#[tauri::command]
pub async fn test_ollama(host: String) -> Result<bool, String> {
    atomic_core::providers::models::test_ollama_connection(&host).await
}

#[tauri::command]
pub async fn get_ollama_models(
    host: String,
) -> Result<Vec<atomic_core::providers::models::OllamaModel>, String> {
    atomic_core::providers::models::fetch_ollama_models(&host).await
}

#[tauri::command]
pub async fn get_ollama_embedding_models_cmd(
    host: String,
) -> Result<Vec<atomic_core::providers::AvailableModel>, String> {
    atomic_core::providers::models::get_ollama_embedding_models(&host).await
}

#[tauri::command]
pub async fn get_ollama_llm_models_cmd(
    host: String,
) -> Result<Vec<atomic_core::providers::AvailableModel>, String> {
    atomic_core::providers::models::get_ollama_llm_models(&host).await
}

#[tauri::command]
pub fn verify_provider_configured(core: State<AtomicCore>) -> Result<bool, String> {
    core.verify_provider_configured().map_err(|e| e.to_string())
}
