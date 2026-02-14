//! Tag CRUD commands — delegates to AtomicCore

use atomic_core::AtomicCore;
use tauri::State;

#[tauri::command]
pub fn get_all_tags(core: State<AtomicCore>) -> Result<Vec<atomic_core::TagWithCount>, String> {
    core.get_all_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(
    core: State<AtomicCore>,
    name: String,
    parent_id: Option<String>,
) -> Result<atomic_core::Tag, String> {
    core.create_tag(&name, parent_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag(
    core: State<AtomicCore>,
    id: String,
    name: String,
    parent_id: Option<String>,
) -> Result<atomic_core::Tag, String> {
    core.update_tag(&id, &name, parent_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(core: State<AtomicCore>, id: String) -> Result<(), String> {
    core.delete_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_atoms_by_tag(
    core: State<AtomicCore>,
    tag_id: String,
) -> Result<Vec<atomic_core::AtomWithTags>, String> {
    core.get_atoms_by_tag(&tag_id).map_err(|e| e.to_string())
}
