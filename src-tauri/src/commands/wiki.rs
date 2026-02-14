//! Wiki article commands — delegates to AtomicCore

use atomic_core::AtomicCore;
use tauri::State;

#[tauri::command]
pub fn get_all_wiki_articles(
    core: State<AtomicCore>,
) -> Result<Vec<atomic_core::WikiArticleSummary>, String> {
    core.get_all_wiki_articles().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_wiki_article(
    core: State<AtomicCore>,
    tag_id: String,
) -> Result<Option<atomic_core::WikiArticleWithCitations>, String> {
    core.get_wiki(&tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_wiki_article_status(
    core: State<AtomicCore>,
    tag_id: String,
) -> Result<atomic_core::WikiArticleStatus, String> {
    core.get_wiki_status(&tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_wiki_article(
    core: State<'_, AtomicCore>,
    tag_id: String,
    tag_name: String,
) -> Result<atomic_core::WikiArticleWithCitations, String> {
    core.generate_wiki(&tag_id, &tag_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_wiki_article(
    core: State<'_, AtomicCore>,
    tag_id: String,
    tag_name: String,
) -> Result<atomic_core::WikiArticleWithCitations, String> {
    // AtomicCore doesn't have update_wiki yet, so we use database-level operations
    // The update_wiki involves prepare_wiki_update + update_wiki_content + save
    let db = core.database();

    let (provider_config, wiki_model, existing, update_input) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let settings_map = atomic_core::settings::get_all_settings(&conn).map_err(|e| e.to_string())?;
        let provider_config = atomic_core::ProviderConfig::from_settings(&settings_map);

        if provider_config.provider_type == atomic_core::ProviderType::OpenRouter
            && provider_config.openrouter_api_key.is_none()
        {
            return Err("OpenRouter API key not configured. Please set it in Settings.".to_string());
        }

        let wiki_model = match provider_config.provider_type {
            atomic_core::ProviderType::Ollama => provider_config.llm_model().to_string(),
            atomic_core::ProviderType::OpenRouter => settings_map
                .get("wiki_model")
                .cloned()
                .unwrap_or_else(|| "anthropic/claude-sonnet-4".to_string()),
        };

        let existing = atomic_core::wiki::load_wiki_article(&conn, &tag_id)?;
        let update_input = if let Some(ref ex) = existing {
            atomic_core::wiki::prepare_wiki_update(&conn, &tag_id, &tag_name, &ex.article, &ex.citations)?
        } else {
            None
        };

        (provider_config, wiki_model, existing, update_input)
    };

    let existing = existing.ok_or("No existing article to update")?;

    let input = match update_input {
        Some(input) => input,
        None => return Ok(existing),
    };

    let result =
        atomic_core::wiki::update_wiki_content(&provider_config, &input, &wiki_model).await?;

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        atomic_core::wiki::save_wiki_article(&conn, &result.article, &result.citations)?;
    }

    Ok(result)
}

#[tauri::command]
pub fn delete_wiki_article(core: State<AtomicCore>, tag_id: String) -> Result<(), String> {
    core.delete_wiki(&tag_id).map_err(|e| e.to_string())
}
