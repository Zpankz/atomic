//! Atom CRUD commands — delegates to AtomicCore

use crate::event_bridge::embedding_event_callback;
use atomic_core::AtomicCore;
use tauri::State;

#[tauri::command]
pub fn get_all_atoms(
    core: State<AtomicCore>,
) -> Result<Vec<atomic_core::AtomWithTags>, String> {
    core.get_all_atoms().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_atom_by_id(
    core: State<AtomicCore>,
    id: String,
) -> Result<Option<atomic_core::AtomWithTags>, String> {
    core.get_atom(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_atom(
    app_handle: tauri::AppHandle,
    core: State<AtomicCore>,
    content: String,
    source_url: Option<String>,
    tag_ids: Vec<String>,
) -> Result<atomic_core::AtomWithTags, String> {
    let request = atomic_core::CreateAtomRequest {
        content,
        source_url,
        tag_ids,
    };
    core.create_atom(request, embedding_event_callback(app_handle))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_atom(
    app_handle: tauri::AppHandle,
    core: State<AtomicCore>,
    id: String,
    content: String,
    source_url: Option<String>,
    tag_ids: Vec<String>,
) -> Result<atomic_core::AtomWithTags, String> {
    let request = atomic_core::UpdateAtomRequest {
        content,
        source_url,
        tag_ids,
    };
    core.update_atom(&id, request, embedding_event_callback(app_handle))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_atom(core: State<AtomicCore>, id: String) -> Result<(), String> {
    core.delete_atom(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_atoms(
    core: State<AtomicCore>,
    tag_id: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<atomic_core::PaginatedAtoms, String> {
    core.list_atoms(tag_id.as_deref(), limit.unwrap_or(50), offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}
