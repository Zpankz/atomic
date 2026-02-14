//! Semantic graph commands — delegates to AtomicCore

use atomic_core::AtomicCore;
use tauri::State;

#[tauri::command]
pub fn get_semantic_edges(
    core: State<AtomicCore>,
    min_similarity: f32,
) -> Result<Vec<atomic_core::SemanticEdge>, String> {
    core.get_semantic_edges(min_similarity)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_atom_neighborhood(
    core: State<AtomicCore>,
    atom_id: String,
    depth: i32,
    min_similarity: f32,
) -> Result<atomic_core::NeighborhoodGraph, String> {
    core.get_atom_neighborhood(&atom_id, depth, min_similarity)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rebuild_semantic_edges(core: State<AtomicCore>) -> Result<i32, String> {
    core.rebuild_semantic_edges().map_err(|e| e.to_string())
}
