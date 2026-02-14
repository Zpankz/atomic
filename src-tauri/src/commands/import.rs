//! Import commands — uses AtomicCore for embedding, raw SQL for import logic

use crate::event_bridge::embedding_event_callback;
use atomic_core::AtomicCore;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// Result of an import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: i32,
    pub skipped: i32,
    pub errors: i32,
    pub tags_created: i32,
    pub tags_linked: i32,
}

/// Progress event payload for import operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgressPayload {
    pub current: i32,
    pub total: i32,
    pub current_file: String,
    pub status: String,
}

#[tauri::command]
pub async fn import_obsidian_vault(
    app: AppHandle,
    core: State<'_, AtomicCore>,
    vault_path: String,
    max_notes: Option<i32>,
) -> Result<ImportResult, String> {
    let vault_path = Path::new(&vault_path);

    if !vault_path.exists() {
        return Err(format!("Vault not found at {:?}", vault_path));
    }

    let vault_name = vault_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Vault".to_string());

    let exclude_patterns: Vec<&str> = atomic_core::import::obsidian::DEFAULT_EXCLUDES.to_vec();
    let mut note_files =
        atomic_core::import::obsidian::discover_notes(vault_path, &exclude_patterns)?;

    if note_files.is_empty() {
        return Ok(ImportResult {
            imported: 0,
            skipped: 0,
            errors: 0,
            tags_created: 0,
            tags_linked: 0,
        });
    }

    if let Some(max) = max_notes {
        note_files.truncate(max as usize);
    }

    let total = note_files.len() as i32;
    let mut stats = ImportResult {
        imported: 0,
        skipped: 0,
        errors: 0,
        tags_created: 0,
        tags_linked: 0,
    };

    let mut tag_cache: HashMap<(String, Option<String>), String> = HashMap::new();
    let mut imported_atoms: Vec<(String, String)> = Vec::new();
    let db = core.database();

    for (index, file_path) in note_files.iter().enumerate() {
        let relative_path = file_path.strip_prefix(vault_path).unwrap_or(file_path);
        let relative_str = relative_path.to_string_lossy().to_string();

        let note = match atomic_core::import::obsidian::parse_obsidian_note(
            file_path,
            relative_path,
            &vault_name,
        ) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error parsing {}: {}", relative_str, e);
                stats.errors += 1;
                let _ = app.emit(
                    "import-progress",
                    ImportProgressPayload {
                        current: index as i32 + 1,
                        total,
                        current_file: relative_str,
                        status: "error".to_string(),
                    },
                );
                continue;
            }
        };

        if note.content.trim().len() < 10 {
            stats.skipped += 1;
            let _ = app.emit(
                "import-progress",
                ImportProgressPayload {
                    current: index as i32 + 1,
                    total,
                    current_file: relative_str,
                    status: "skipped".to_string(),
                },
            );
            continue;
        }

        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM atoms WHERE source_url = ?1 LIMIT 1",
                [&note.source_url],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if exists {
            stats.skipped += 1;
            let _ = app.emit(
                "import-progress",
                ImportProgressPayload {
                    current: index as i32 + 1,
                    total,
                    current_file: relative_str,
                    status: "skipped".to_string(),
                },
            );
            drop(conn);
            continue;
        }

        let atom_id = Uuid::new_v4().to_string();
        match conn.execute(
            "INSERT INTO atoms (id, content, source_url, created_at, updated_at, embedding_status, tagging_status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending', 'pending')",
            params![
                &atom_id,
                &note.content,
                &note.source_url,
                &note.created_at,
                &note.updated_at,
            ],
        ) {
            Ok(_) => {
                imported_atoms.push((atom_id.clone(), note.content.clone()));
            }
            Err(e) => {
                eprintln!("Error inserting atom for {}: {}", relative_str, e);
                stats.errors += 1;
                let _ = app.emit(
                    "import-progress",
                    ImportProgressPayload {
                        current: index as i32 + 1,
                        total,
                        current_file: relative_str,
                        status: "error".to_string(),
                    },
                );
                drop(conn);
                continue;
            }
        }

        // Helper closure to get or create a tag
        let get_or_create_tag = |conn: &rusqlite::Connection,
                                  tag_cache: &mut HashMap<(String, Option<String>), String>,
                                  name: &str,
                                  parent_id: Option<&str>,
                                  stats: &mut ImportResult|
         -> Option<String> {
            let cache_key = (name.to_lowercase(), parent_id.map(|s| s.to_string()));

            if let Some(cached_id) = tag_cache.get(&cache_key) {
                return Some(cached_id.clone());
            }

            let existing: Option<String> = if let Some(pid) = parent_id {
                conn.query_row(
                    "SELECT id FROM tags WHERE LOWER(name) = LOWER(?1) AND parent_id = ?2 LIMIT 1",
                    params![name, pid],
                    |row| row.get(0),
                )
                .ok()
            } else {
                conn.query_row(
                    "SELECT id FROM tags WHERE LOWER(name) = LOWER(?1) AND parent_id IS NULL LIMIT 1",
                    [name],
                    |row| row.get(0),
                )
                .ok()
            };

            let id = match existing {
                Some(id) => id,
                None => {
                    let new_id = Uuid::new_v4().to_string();
                    let now = Utc::now().to_rfc3339();
                    if let Err(e) = conn.execute(
                        "INSERT INTO tags (id, name, parent_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                        params![&new_id, name, parent_id, &now],
                    ) {
                        eprintln!("Error creating tag '{}': {}", name, e);
                        return None;
                    }
                    stats.tags_created += 1;
                    new_id
                }
            };

            tag_cache.insert(cache_key, id.clone());
            Some(id)
        };

        // Process hierarchical folder tags
        let mut folder_tag_ids: Vec<String> = Vec::new();
        for htag in &note.folder_tags {
            let parent_id = if htag.parent_path.is_empty() {
                None
            } else {
                let parent_index = htag.parent_path.len() - 1;
                folder_tag_ids.get(parent_index).map(|s| s.as_str())
            };

            if let Some(tag_id) =
                get_or_create_tag(&conn, &mut tag_cache, &htag.name, parent_id, &mut stats)
            {
                folder_tag_ids.push(tag_id.clone());
                if let Err(e) = conn.execute(
                    "INSERT OR IGNORE INTO atom_tags (atom_id, tag_id) VALUES (?1, ?2)",
                    params![&atom_id, &tag_id],
                ) {
                    eprintln!("Error linking folder tag '{}' to atom: {}", htag.name, e);
                    continue;
                }
                stats.tags_linked += 1;
            }
        }

        // Process flat frontmatter tags
        for tag_name in &note.frontmatter_tags {
            if let Some(tag_id) =
                get_or_create_tag(&conn, &mut tag_cache, tag_name, None, &mut stats)
            {
                if let Err(e) = conn.execute(
                    "INSERT OR IGNORE INTO atom_tags (atom_id, tag_id) VALUES (?1, ?2)",
                    params![&atom_id, &tag_id],
                ) {
                    eprintln!("Error linking tag '{}' to atom: {}", tag_name, e);
                    continue;
                }
                stats.tags_linked += 1;
            }
        }

        stats.imported += 1;
        let _ = app.emit(
            "import-progress",
            ImportProgressPayload {
                current: index as i32 + 1,
                total,
                current_file: relative_str,
                status: "importing".to_string(),
            },
        );

        drop(conn);
    }

    // Trigger embedding processing for all imported atoms
    if !imported_atoms.is_empty() {
        {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            for (atom_id, _) in &imported_atoms {
                let _ = conn.execute(
                    "UPDATE atoms SET embedding_status = 'processing' WHERE id = ?1",
                    [atom_id],
                );
            }
        }

        let on_event = embedding_event_callback(app.clone());
        let db_clone = core.database();
        tokio::spawn(async move {
            atomic_core::embedding::process_embedding_batch(
                db_clone,
                imported_atoms,
                false,
                on_event,
            )
            .await;
        });
    }

    Ok(stats)
}
