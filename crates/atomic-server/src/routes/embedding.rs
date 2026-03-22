//! Embedding management routes

use crate::db_extractor::Db;
use crate::event_bridge::embedding_event_callback;
use crate::state::AppState;
use actix_web::{web, HttpResponse};

pub async fn process_pending_embeddings(state: web::Data<AppState>, db: Db) -> HttpResponse {
    let on_event = embedding_event_callback(state.event_tx.clone());
    match db.0.process_pending_embeddings(on_event) {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({"count": count})),
        Err(e) => crate::error::error_response(e),
    }
}

pub async fn process_pending_tagging(state: web::Data<AppState>, db: Db) -> HttpResponse {
    let on_event = embedding_event_callback(state.event_tx.clone());
    match db.0.process_pending_tagging(on_event) {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({"count": count})),
        Err(e) => crate::error::error_response(e),
    }
}

pub async fn retry_embedding(
    state: web::Data<AppState>,
    db: Db,
    path: web::Path<String>,
) -> HttpResponse {
    let atom_id = path.into_inner();
    let on_event = embedding_event_callback(state.event_tx.clone());
    match db.0.retry_embedding(&atom_id, on_event) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"status": "ok"})),
        Err(e) => crate::error::error_response(e),
    }
}

pub async fn retry_tagging(
    state: web::Data<AppState>,
    db: Db,
    path: web::Path<String>,
) -> HttpResponse {
    let atom_id = path.into_inner();
    let on_event = embedding_event_callback(state.event_tx.clone());
    match db.0.retry_tagging(&atom_id, on_event) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"status": "ok"})),
        Err(e) => crate::error::error_response(e),
    }
}

pub async fn reset_stuck_processing(db: Db) -> HttpResponse {
    match db.0.reset_stuck_processing() {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({"count": count})),
        Err(e) => crate::error::error_response(e),
    }
}

pub async fn get_embedding_status(
    db: Db,
    path: web::Path<String>,
) -> HttpResponse {
    let atom_id = path.into_inner();
    match db.0.get_embedding_status(&atom_id) {
        Ok(status) => HttpResponse::Ok().json(serde_json::json!({"status": status})),
        Err(e) => crate::error::error_response(e),
    }
}
