//! Chat and agent commands — delegates to AtomicCore

use crate::event_bridge::chat_event_callback;
use atomic_core::AtomicCore;
use tauri::State;

#[tauri::command]
pub fn create_conversation(
    core: State<AtomicCore>,
    tag_ids: Vec<String>,
    title: Option<String>,
) -> Result<atomic_core::ConversationWithTags, String> {
    core.create_conversation(&tag_ids, title.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_conversations(
    core: State<AtomicCore>,
    filter_tag_id: Option<String>,
    limit: i32,
    offset: i32,
) -> Result<Vec<atomic_core::ConversationWithTags>, String> {
    core.get_conversations(filter_tag_id.as_deref(), limit, offset)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_conversation(
    core: State<AtomicCore>,
    conversation_id: String,
) -> Result<Option<atomic_core::ConversationWithMessages>, String> {
    core.get_conversation(&conversation_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_conversation(
    core: State<AtomicCore>,
    id: String,
    title: Option<String>,
    is_archived: Option<bool>,
) -> Result<atomic_core::Conversation, String> {
    core.update_conversation(&id, title.as_deref(), is_archived)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_conversation(core: State<AtomicCore>, id: String) -> Result<(), String> {
    core.delete_conversation(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_conversation_scope(
    core: State<AtomicCore>,
    conversation_id: String,
    tag_ids: Vec<String>,
) -> Result<atomic_core::ConversationWithTags, String> {
    core.set_conversation_scope(&conversation_id, &tag_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag_to_scope(
    core: State<AtomicCore>,
    conversation_id: String,
    tag_id: String,
) -> Result<atomic_core::ConversationWithTags, String> {
    core.add_tag_to_scope(&conversation_id, &tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag_from_scope(
    core: State<AtomicCore>,
    conversation_id: String,
    tag_id: String,
) -> Result<atomic_core::ConversationWithTags, String> {
    core.remove_tag_from_scope(&conversation_id, &tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_chat_message(
    app_handle: tauri::AppHandle,
    core: State<'_, AtomicCore>,
    conversation_id: String,
    content: String,
) -> Result<atomic_core::ChatMessageWithContext, String> {
    core.send_chat_message(&conversation_id, &content, chat_event_callback(app_handle))
        .await
        .map_err(|e| e.to_string())
}
