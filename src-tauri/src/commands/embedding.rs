//! Embedding and search commands — delegates to AtomicCore

use crate::event_bridge::embedding_event_callback;
use atomic_core::AtomicCore;
use tauri::State;

#[tauri::command]
pub fn find_similar_atoms(
    core: State<AtomicCore>,
    atom_id: String,
    limit: i32,
    threshold: f32,
) -> Result<Vec<atomic_core::SimilarAtomResult>, String> {
    core.find_similar(&atom_id, limit, threshold)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_atoms_semantic(
    core: State<'_, AtomicCore>,
    query: String,
    limit: i32,
    threshold: f32,
) -> Result<Vec<atomic_core::SemanticSearchResult>, String> {
    let options =
        atomic_core::SearchOptions::new(query, atomic_core::SearchMode::Semantic, limit)
            .with_threshold(threshold);
    core.search(options).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_atoms_keyword(
    core: State<'_, AtomicCore>,
    query: String,
    limit: i32,
) -> Result<Vec<atomic_core::SemanticSearchResult>, String> {
    let options =
        atomic_core::SearchOptions::new(query, atomic_core::SearchMode::Keyword, limit);
    core.search(options).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_atoms_hybrid(
    core: State<'_, AtomicCore>,
    query: String,
    limit: i32,
    threshold: f32,
) -> Result<Vec<atomic_core::SemanticSearchResult>, String> {
    let options =
        atomic_core::SearchOptions::new(query, atomic_core::SearchMode::Hybrid, limit)
            .with_threshold(threshold);
    core.search(options).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn retry_embedding(
    app_handle: tauri::AppHandle,
    core: State<AtomicCore>,
    atom_id: String,
) -> Result<(), String> {
    core.retry_embedding(&atom_id, embedding_event_callback(app_handle))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_stuck_processing(core: State<AtomicCore>) -> Result<i32, String> {
    core.reset_stuck_processing().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn process_pending_embeddings(
    app_handle: tauri::AppHandle,
    core: State<'_, AtomicCore>,
) -> Result<i32, String> {
    core.process_pending_embeddings(embedding_event_callback(app_handle))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn process_pending_tagging(
    app_handle: tauri::AppHandle,
    core: State<'_, AtomicCore>,
) -> Result<i32, String> {
    core.process_pending_tagging(embedding_event_callback(app_handle))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_embedding_status(
    core: State<AtomicCore>,
    atom_id: String,
) -> Result<String, String> {
    core.get_embedding_status(&atom_id).map_err(|e| e.to_string())
}
