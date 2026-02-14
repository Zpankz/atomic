//! Data models for Atomic Tauri app
//!
//! This module defines Tauri-specific types and conversions.

use serde::{Deserialize, Serialize};

/// Request payload for creating an atom (used by both Tauri commands and HTTP API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAtomRequest {
    pub content: String,
    pub source_url: Option<String>,
    pub tag_ids: Vec<String>,
}

impl From<CreateAtomRequest> for atomic_core::CreateAtomRequest {
    fn from(req: CreateAtomRequest) -> Self {
        atomic_core::CreateAtomRequest {
            content: req.content,
            source_url: req.source_url,
            tag_ids: req.tag_ids,
        }
    }
}
