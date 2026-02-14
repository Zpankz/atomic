//! Bridge between atomic-core callback events and Tauri events
//!
//! Provides helper functions that create callback closures which forward
//! EmbeddingEvent and ChatEvent instances into Tauri app_handle.emit() calls.

use atomic_core::agent::ChatEvent;
use atomic_core::models::{EmbeddingCompletePayload, TaggingCompletePayload};
use atomic_core::EmbeddingEvent;
use tauri::{AppHandle, Emitter};

/// Create an EmbeddingEvent callback that emits Tauri events
pub fn embedding_event_callback(
    app_handle: AppHandle,
) -> impl Fn(EmbeddingEvent) + Send + Sync + Clone + 'static {
    move |event: EmbeddingEvent| match event {
        EmbeddingEvent::Started { .. } => {}
        EmbeddingEvent::EmbeddingComplete { atom_id } => {
            let _ = app_handle.emit(
                "embedding-complete",
                EmbeddingCompletePayload {
                    atom_id,
                    status: "complete".to_string(),
                    error: None,
                },
            );
        }
        EmbeddingEvent::EmbeddingFailed { atom_id, error } => {
            let _ = app_handle.emit(
                "embedding-complete",
                EmbeddingCompletePayload {
                    atom_id,
                    status: "failed".to_string(),
                    error: Some(error),
                },
            );
        }
        EmbeddingEvent::TaggingComplete {
            atom_id,
            tags_extracted,
            new_tags_created,
        } => {
            let _ = app_handle.emit(
                "tagging-complete",
                TaggingCompletePayload {
                    atom_id,
                    status: "complete".to_string(),
                    error: None,
                    tags_extracted,
                    new_tags_created,
                },
            );
        }
        EmbeddingEvent::TaggingFailed { atom_id, error } => {
            let _ = app_handle.emit(
                "tagging-complete",
                TaggingCompletePayload {
                    atom_id,
                    status: "failed".to_string(),
                    error: Some(error),
                    tags_extracted: vec![],
                    new_tags_created: vec![],
                },
            );
        }
        EmbeddingEvent::TaggingSkipped { atom_id } => {
            let _ = app_handle.emit(
                "tagging-complete",
                TaggingCompletePayload {
                    atom_id,
                    status: "skipped".to_string(),
                    error: None,
                    tags_extracted: vec![],
                    new_tags_created: vec![],
                },
            );
        }
    }
}

/// Create a ChatEvent callback that emits Tauri events
pub fn chat_event_callback(
    app_handle: AppHandle,
) -> impl Fn(ChatEvent) + Send + Sync + 'static {
    move |event: ChatEvent| match event {
        ChatEvent::StreamDelta {
            conversation_id,
            content,
        } => {
            let _ = app_handle.emit(
                "chat-stream-delta",
                serde_json::json!({
                    "conversation_id": conversation_id,
                    "content": content,
                }),
            );
        }
        ChatEvent::ToolStart {
            conversation_id,
            tool_call_id,
            tool_name,
            tool_input,
        } => {
            let _ = app_handle.emit(
                "chat-tool-start",
                serde_json::json!({
                    "conversation_id": conversation_id,
                    "tool_call_id": tool_call_id,
                    "tool_name": tool_name,
                    "tool_input": tool_input,
                }),
            );
        }
        ChatEvent::ToolComplete {
            conversation_id,
            tool_call_id,
            results_count,
        } => {
            let _ = app_handle.emit(
                "chat-tool-complete",
                serde_json::json!({
                    "conversation_id": conversation_id,
                    "tool_call_id": tool_call_id,
                    "results_count": results_count,
                }),
            );
        }
        ChatEvent::Complete {
            conversation_id,
            message,
        } => {
            let _ = app_handle.emit(
                "chat-complete",
                serde_json::json!({
                    "conversation_id": conversation_id,
                    "message": message,
                }),
            );
        }
        ChatEvent::Error {
            conversation_id,
            error,
        } => {
            let _ = app_handle.emit(
                "chat-error",
                serde_json::json!({
                    "conversation_id": conversation_id,
                    "error": error,
                }),
            );
        }
    }
}
