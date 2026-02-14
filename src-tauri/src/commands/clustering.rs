//! Clustering commands — delegates to AtomicCore

use atomic_core::AtomicCore;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn compute_clusters(
    core: State<AtomicCore>,
    min_similarity: Option<f32>,
    min_cluster_size: Option<i32>,
) -> Result<Vec<atomic_core::AtomCluster>, String> {
    let threshold = min_similarity.unwrap_or(0.5);
    let min_size = min_cluster_size.unwrap_or(2);
    let clusters = core
        .compute_clusters(threshold, min_size)
        .map_err(|e| e.to_string())?;
    core.save_clusters(&clusters).map_err(|e| e.to_string())?;
    Ok(clusters)
}

#[tauri::command]
pub fn get_clusters(core: State<AtomicCore>) -> Result<Vec<atomic_core::AtomCluster>, String> {
    core.get_clusters().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_connection_counts(
    core: State<AtomicCore>,
    min_similarity: Option<f32>,
) -> Result<HashMap<String, i32>, String> {
    let threshold = min_similarity.unwrap_or(0.5);
    core.get_connection_counts(threshold)
        .map_err(|e| e.to_string())
}
