//! Storage abstraction layer for atomic-core.
//!
//! This module defines the trait hierarchy for database backends and provides
//! the default SQLite implementation. Alternative backends (e.g., Postgres)
//! can be added by implementing the `Storage` supertrait.

pub mod traits;
pub mod sqlite;
pub mod postgres;

pub use traits::*;
pub use sqlite::SqliteStorage;

#[cfg(feature = "postgres")]
pub use postgres::PostgresStorage;

use crate::error::AtomicCoreError;

/// Runtime-dispatched storage backend.
///
/// AtomicCore holds this enum to support both SQLite and Postgres at runtime.
/// For SQLite, sync helper methods are called directly.
/// For Postgres, async trait methods are bridged to sync via `block_on`.
#[derive(Clone)]
pub enum StorageBackend {
    Sqlite(SqliteStorage),
    #[cfg(feature = "postgres")]
    Postgres(PostgresStorage),
}

impl StorageBackend {
    /// Get the underlying SqliteStorage, if this is a SQLite backend.
    /// Used for operations not yet abstracted behind the storage trait
    /// (e.g., embedding pipeline internals that directly use `Arc<Database>`).
    pub(crate) fn as_sqlite(&self) -> Option<&SqliteStorage> {
        match self {
            StorageBackend::Sqlite(s) => Some(s),
            #[cfg(feature = "postgres")]
            StorageBackend::Postgres(_) => None,
        }
    }

    /// Get the database path (for display).
    pub(crate) fn storage_path(&self) -> &std::path::Path {
        match self {
            StorageBackend::Sqlite(s) => &s.db.db_path,
            #[cfg(feature = "postgres")]
            StorageBackend::Postgres(_) => std::path::Path::new("postgres"),
        }
    }
}

/// Helper: bridge an async call to sync using the current tokio Handle.
#[cfg(feature = "postgres")]
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Handle::current().block_on(f)
}

// ==================== Sync dispatch methods ====================
//
// Each method dispatches to either the SqliteStorage sync helper
// or the PostgresStorage async trait method (via block_on).
// This keeps AtomicCore's public API synchronous while supporting
// both backends at runtime.

/// Macro to generate dispatch methods. For each method:
/// - SQLite: calls `s.$sqlite_method($($arg),*)`
/// - Postgres: calls `block_on(<TraitName>::$trait_method(s, $($arg),*))`
macro_rules! dispatch {
    (
        $(
            fn $name:ident(&self $(, $arg:ident: $argty:ty)*) -> $ret:ty
                => sqlite: $sqlite_method:ident, pg_trait: $trait_name:path, pg_method: $pg_method:ident;
        )*
    ) => {
        impl StorageBackend {
            $(
                pub(crate) fn $name(&self $(, $arg: $argty)*) -> $ret {
                    match self {
                        StorageBackend::Sqlite(s) => s.$sqlite_method($($arg),*),
                        #[cfg(feature = "postgres")]
                        StorageBackend::Postgres(s) => {
                            block_on(<PostgresStorage as $trait_name>::$pg_method(s $(, $arg)*))
                        }
                    }
                }
            )*
        }
    };
}

use crate::compaction::{CompactionResult, TagMerge};
use crate::models::*;
use crate::{CreateAtomRequest, ListAtomsParams, UpdateAtomRequest};
use std::collections::HashMap;

dispatch! {
    // ---- AtomStore ----
    fn get_all_atoms_impl(&self) -> Result<Vec<AtomWithTags>, AtomicCoreError>
        => sqlite: get_all_atoms_impl, pg_trait: AtomStore, pg_method: get_all_atoms;
    fn get_atom_impl(&self, id: &str) -> Result<Option<AtomWithTags>, AtomicCoreError>
        => sqlite: get_atom_impl, pg_trait: AtomStore, pg_method: get_atom;
    fn insert_atom_impl(&self, id: &str, request: &CreateAtomRequest, created_at: &str) -> Result<AtomWithTags, AtomicCoreError>
        => sqlite: insert_atom_impl, pg_trait: AtomStore, pg_method: insert_atom;
    fn insert_atoms_bulk_impl(&self, atoms: &[(String, CreateAtomRequest, String)]) -> Result<Vec<AtomWithTags>, AtomicCoreError>
        => sqlite: insert_atoms_bulk_impl, pg_trait: AtomStore, pg_method: insert_atoms_bulk;
    fn update_atom_impl(&self, id: &str, request: &UpdateAtomRequest, updated_at: &str) -> Result<AtomWithTags, AtomicCoreError>
        => sqlite: update_atom_impl, pg_trait: AtomStore, pg_method: update_atom;
    fn delete_atom_impl(&self, id: &str) -> Result<(), AtomicCoreError>
        => sqlite: delete_atom_impl, pg_trait: AtomStore, pg_method: delete_atom;
    fn get_atoms_by_tag_impl(&self, tag_id: &str) -> Result<Vec<AtomWithTags>, AtomicCoreError>
        => sqlite: get_atoms_by_tag_impl, pg_trait: AtomStore, pg_method: get_atoms_by_tag;
    fn list_atoms_impl(&self, params: &ListAtomsParams) -> Result<PaginatedAtoms, AtomicCoreError>
        => sqlite: list_atoms_impl, pg_trait: AtomStore, pg_method: list_atoms;
    fn get_source_list_impl(&self) -> Result<Vec<SourceInfo>, AtomicCoreError>
        => sqlite: get_source_list_impl, pg_trait: AtomStore, pg_method: get_source_list;
    fn get_embedding_status_impl(&self, atom_id: &str) -> Result<String, AtomicCoreError>
        => sqlite: get_embedding_status_impl, pg_trait: AtomStore, pg_method: get_embedding_status;
    fn get_atom_positions_impl(&self) -> Result<Vec<AtomPosition>, AtomicCoreError>
        => sqlite: get_atom_positions_impl, pg_trait: AtomStore, pg_method: get_atom_positions;
    fn save_atom_positions_impl(&self, positions: &[AtomPosition]) -> Result<(), AtomicCoreError>
        => sqlite: save_atom_positions_impl, pg_trait: AtomStore, pg_method: save_atom_positions;
    fn get_atoms_with_embeddings_impl(&self) -> Result<Vec<AtomWithEmbedding>, AtomicCoreError>
        => sqlite: get_atoms_with_embeddings_impl, pg_trait: AtomStore, pg_method: get_atoms_with_embeddings;

    // ---- TagStore ----
    fn get_all_tags_impl(&self) -> Result<Vec<TagWithCount>, AtomicCoreError>
        => sqlite: get_all_tags_impl, pg_trait: TagStore, pg_method: get_all_tags;
    fn get_all_tags_filtered_impl(&self, min_count: i32) -> Result<Vec<TagWithCount>, AtomicCoreError>
        => sqlite: get_all_tags_filtered_impl, pg_trait: TagStore, pg_method: get_all_tags_filtered;
    fn get_tag_children_impl(&self, parent_id: &str, min_count: i32, limit: i32, offset: i32) -> Result<PaginatedTagChildren, AtomicCoreError>
        => sqlite: get_tag_children_impl, pg_trait: TagStore, pg_method: get_tag_children;
    fn create_tag_impl(&self, name: &str, parent_id: Option<&str>) -> Result<Tag, AtomicCoreError>
        => sqlite: create_tag_impl, pg_trait: TagStore, pg_method: create_tag;
    fn update_tag_impl(&self, id: &str, name: &str, parent_id: Option<&str>) -> Result<Tag, AtomicCoreError>
        => sqlite: update_tag_impl, pg_trait: TagStore, pg_method: update_tag;
    fn delete_tag_impl(&self, id: &str, recursive: bool) -> Result<(), AtomicCoreError>
        => sqlite: delete_tag_impl, pg_trait: TagStore, pg_method: delete_tag;
    fn get_related_tags_impl(&self, tag_id: &str, limit: usize) -> Result<Vec<RelatedTag>, AtomicCoreError>
        => sqlite: get_related_tags_impl, pg_trait: TagStore, pg_method: get_related_tags;
    fn get_tags_for_compaction_impl(&self) -> Result<String, AtomicCoreError>
        => sqlite: get_tags_for_compaction_impl, pg_trait: TagStore, pg_method: get_tags_for_compaction;
    fn apply_tag_merges_impl(&self, merges: &[TagMerge]) -> Result<CompactionResult, AtomicCoreError>
        => sqlite: apply_tag_merges_impl, pg_trait: TagStore, pg_method: apply_tag_merges;

    // ---- ChunkStore ----
    fn reset_stuck_processing_sync(&self) -> Result<i32, AtomicCoreError>
        => sqlite: reset_stuck_processing_sync, pg_trait: ChunkStore, pg_method: reset_stuck_processing;
    fn rebuild_semantic_edges_sync(&self) -> Result<i32, AtomicCoreError>
        => sqlite: rebuild_semantic_edges_sync, pg_trait: ChunkStore, pg_method: rebuild_semantic_edges;
    fn get_semantic_edges_sync(&self, min_similarity: f32) -> Result<Vec<SemanticEdge>, AtomicCoreError>
        => sqlite: get_semantic_edges_sync, pg_trait: ChunkStore, pg_method: get_semantic_edges;
    fn get_atom_neighborhood_sync(&self, atom_id: &str, depth: i32, min_similarity: f32) -> Result<NeighborhoodGraph, AtomicCoreError>
        => sqlite: get_atom_neighborhood_sync, pg_trait: ChunkStore, pg_method: get_atom_neighborhood;
    fn get_connection_counts_sync(&self, min_similarity: f32) -> Result<HashMap<String, i32>, AtomicCoreError>
        => sqlite: get_connection_counts_sync, pg_trait: ChunkStore, pg_method: get_connection_counts;
    fn recompute_all_tag_embeddings_sync(&self) -> Result<i32, AtomicCoreError>
        => sqlite: recompute_all_tag_embeddings_sync, pg_trait: ChunkStore, pg_method: recompute_all_tag_embeddings;
    fn check_vector_extension_sync(&self) -> Result<String, AtomicCoreError>
        => sqlite: check_vector_extension_sync, pg_trait: ChunkStore, pg_method: check_vector_extension;

    // ---- SearchStore ----
    fn vector_search_sync(&self, query_embedding: &[f32], limit: i32, threshold: f32, tag_id: Option<&str>) -> Result<Vec<SemanticSearchResult>, AtomicCoreError>
        => sqlite: vector_search_sync, pg_trait: SearchStore, pg_method: vector_search;
    fn keyword_search_sync(&self, query: &str, limit: i32, tag_id: Option<&str>) -> Result<Vec<SemanticSearchResult>, AtomicCoreError>
        => sqlite: keyword_search_sync, pg_trait: SearchStore, pg_method: keyword_search;
    fn find_similar_sync(&self, atom_id: &str, limit: i32, threshold: f32) -> Result<Vec<SimilarAtomResult>, AtomicCoreError>
        => sqlite: find_similar_sync, pg_trait: SearchStore, pg_method: find_similar;

    // ---- ChatStore ----
    fn create_conversation_sync(&self, tag_ids: &[String], title: Option<&str>) -> Result<ConversationWithTags, AtomicCoreError>
        => sqlite: create_conversation_sync, pg_trait: ChatStore, pg_method: create_conversation;
    fn get_conversations_sync(&self, filter_tag_id: Option<&str>, limit: i32, offset: i32) -> Result<Vec<ConversationWithTags>, AtomicCoreError>
        => sqlite: get_conversations_sync, pg_trait: ChatStore, pg_method: get_conversations;
    fn get_conversation_sync(&self, conversation_id: &str) -> Result<Option<ConversationWithMessages>, AtomicCoreError>
        => sqlite: get_conversation_sync, pg_trait: ChatStore, pg_method: get_conversation;
    fn update_conversation_sync(&self, id: &str, title: Option<&str>, is_archived: Option<bool>) -> Result<Conversation, AtomicCoreError>
        => sqlite: update_conversation_sync, pg_trait: ChatStore, pg_method: update_conversation;
    fn delete_conversation_sync(&self, id: &str) -> Result<(), AtomicCoreError>
        => sqlite: delete_conversation_sync, pg_trait: ChatStore, pg_method: delete_conversation;
    fn set_conversation_scope_sync(&self, conversation_id: &str, tag_ids: &[String]) -> Result<ConversationWithTags, AtomicCoreError>
        => sqlite: set_conversation_scope_sync, pg_trait: ChatStore, pg_method: set_conversation_scope;
    fn add_tag_to_scope_sync(&self, conversation_id: &str, tag_id: &str) -> Result<ConversationWithTags, AtomicCoreError>
        => sqlite: add_tag_to_scope_sync, pg_trait: ChatStore, pg_method: add_tag_to_scope;
    fn remove_tag_from_scope_sync(&self, conversation_id: &str, tag_id: &str) -> Result<ConversationWithTags, AtomicCoreError>
        => sqlite: remove_tag_from_scope_sync, pg_trait: ChatStore, pg_method: remove_tag_from_scope;

    // ---- WikiStore ----
    fn get_wiki_sync(&self, tag_id: &str) -> Result<Option<WikiArticleWithCitations>, AtomicCoreError>
        => sqlite: get_wiki_sync, pg_trait: WikiStore, pg_method: get_wiki;
    fn get_wiki_status_sync(&self, tag_id: &str) -> Result<WikiArticleStatus, AtomicCoreError>
        => sqlite: get_wiki_status_sync, pg_trait: WikiStore, pg_method: get_wiki_status;
    fn delete_wiki_sync(&self, tag_id: &str) -> Result<(), AtomicCoreError>
        => sqlite: delete_wiki_sync, pg_trait: WikiStore, pg_method: delete_wiki;
    fn get_wiki_links_sync(&self, tag_id: &str) -> Result<Vec<WikiLink>, AtomicCoreError>
        => sqlite: get_wiki_links_sync, pg_trait: WikiStore, pg_method: get_wiki_links;
    fn list_wiki_versions_sync(&self, tag_id: &str) -> Result<Vec<WikiVersionSummary>, AtomicCoreError>
        => sqlite: list_wiki_versions_sync, pg_trait: WikiStore, pg_method: list_wiki_versions;
    fn get_wiki_version_sync(&self, version_id: &str) -> Result<Option<WikiArticleVersion>, AtomicCoreError>
        => sqlite: get_wiki_version_sync, pg_trait: WikiStore, pg_method: get_wiki_version;
    fn get_all_wiki_articles_sync(&self) -> Result<Vec<WikiArticleSummary>, AtomicCoreError>
        => sqlite: get_all_wiki_articles_sync, pg_trait: WikiStore, pg_method: get_all_wiki_articles;
    fn get_suggested_wiki_articles_sync(&self, limit: i32) -> Result<Vec<SuggestedArticle>, AtomicCoreError>
        => sqlite: get_suggested_wiki_articles_sync, pg_trait: WikiStore, pg_method: get_suggested_wiki_articles;

    // ---- FeedStore ----
    fn list_feeds_sync(&self) -> Result<Vec<Feed>, AtomicCoreError>
        => sqlite: list_feeds_sync, pg_trait: FeedStore, pg_method: list_feeds;
    fn get_feed_sync(&self, id: &str) -> Result<Feed, AtomicCoreError>
        => sqlite: get_feed_sync, pg_trait: FeedStore, pg_method: get_feed;
    fn update_feed_sync(&self, id: &str, title: Option<&str>, poll_interval: Option<i32>, is_paused: Option<bool>, tag_ids: Option<&[String]>) -> Result<Feed, AtomicCoreError>
        => sqlite: update_feed_sync, pg_trait: FeedStore, pg_method: update_feed;
    fn delete_feed_sync(&self, id: &str) -> Result<(), AtomicCoreError>
        => sqlite: delete_feed_sync, pg_trait: FeedStore, pg_method: delete_feed;
    fn claim_feed_item_sync(&self, feed_id: &str, guid: &str) -> Result<bool, AtomicCoreError>
        => sqlite: claim_feed_item_sync, pg_trait: FeedStore, pg_method: claim_feed_item;
    fn complete_feed_item_sync(&self, feed_id: &str, guid: &str, atom_id: &str) -> Result<(), AtomicCoreError>
        => sqlite: complete_feed_item_sync, pg_trait: FeedStore, pg_method: complete_feed_item;
    fn mark_feed_item_skipped_sync(&self, feed_id: &str, guid: &str, reason: &str) -> Result<(), AtomicCoreError>
        => sqlite: mark_feed_item_skipped_sync, pg_trait: FeedStore, pg_method: mark_feed_item_skipped;

    // ---- ClusterStore ----
    fn compute_clusters_sync(&self, min_similarity: f32, min_cluster_size: i32) -> Result<Vec<AtomCluster>, AtomicCoreError>
        => sqlite: compute_clusters_sync, pg_trait: ClusterStore, pg_method: compute_clusters;
    fn save_clusters_sync(&self, clusters: &[AtomCluster]) -> Result<(), AtomicCoreError>
        => sqlite: save_clusters_sync, pg_trait: ClusterStore, pg_method: save_clusters;
    fn get_clusters_sync(&self) -> Result<Vec<AtomCluster>, AtomicCoreError>
        => sqlite: get_clusters_sync, pg_trait: ClusterStore, pg_method: get_clusters;
    fn get_canvas_level_sync(&self, parent_id: Option<&str>, children_hint: Option<Vec<String>>) -> Result<CanvasLevel, AtomicCoreError>
        => sqlite: get_canvas_level_sync, pg_trait: ClusterStore, pg_method: get_canvas_level;
}
