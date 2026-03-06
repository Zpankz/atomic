//! Database management routes

use crate::state::AppState;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

/// GET /api/databases — List all databases and which is active
pub async fn list_databases(state: web::Data<AppState>) -> HttpResponse {
    match state.manager.list_databases() {
        Ok((databases, active_id)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "databases": databases,
                "active_id": active_id,
            }))
        }
        Err(e) => crate::error::error_response(e),
    }
}

#[derive(Deserialize)]
pub struct CreateDatabaseBody {
    pub name: String,
}

/// POST /api/databases — Create a new database
pub async fn create_database(
    state: web::Data<AppState>,
    body: web::Json<CreateDatabaseBody>,
) -> HttpResponse {
    let name = body.into_inner().name;
    match state.manager.create_database(&name) {
        Ok(info) => HttpResponse::Created().json(info),
        Err(e) => crate::error::error_response(e),
    }
}

#[derive(Deserialize)]
pub struct RenameDatabaseBody {
    pub name: String,
}

/// PUT /api/databases/{id} — Rename a database
pub async fn rename_database(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<RenameDatabaseBody>,
) -> HttpResponse {
    let id = path.into_inner();
    let name = body.into_inner().name;
    match state.manager.registry().rename_database(&id, &name) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"renamed": true})),
        Err(e) => crate::error::error_response(e),
    }
}

/// DELETE /api/databases/{id} — Delete a database (400 if default)
pub async fn delete_database(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let id = path.into_inner();
    match state.manager.delete_database(&id) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Err(e) => crate::error::error_response(e),
    }
}

/// PUT /api/databases/{id}/activate — Switch active database
pub async fn activate_database(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let id = path.into_inner();
    match state.manager.set_active(&id) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"activated": true})),
        Err(e) => crate::error::error_response(e),
    }
}
