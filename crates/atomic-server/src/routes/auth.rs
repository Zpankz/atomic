//! Token management endpoints

use crate::state::AppState;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTokenBody {
    pub name: String,
}

/// POST /api/auth/tokens — Create a new named API token
pub async fn create_token(
    state: web::Data<AppState>,
    body: web::Json<CreateTokenBody>,
) -> HttpResponse {
    let name = body.into_inner().name;
    let registry = state.manager.registry().clone();
    match web::block(move || registry.create_api_token(&name)).await {
        Ok(Ok((info, raw_token))) => HttpResponse::Created().json(serde_json::json!({
            "id": info.id,
            "name": info.name,
            "token": raw_token,
            "prefix": info.token_prefix,
            "created_at": info.created_at,
        })),
        Ok(Err(e)) => crate::error::error_response(e),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

/// GET /api/auth/tokens — List all tokens (metadata only, no raw token values)
pub async fn list_tokens(state: web::Data<AppState>) -> HttpResponse {
    let registry = state.manager.registry().clone();
    match web::block(move || registry.list_api_tokens()).await {
        Ok(Ok(tokens)) => HttpResponse::Ok().json(tokens),
        Ok(Err(e)) => crate::error::error_response(e),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

/// DELETE /api/auth/tokens/{id} — Revoke a token
pub async fn revoke_token(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let token_id = path.into_inner();
    let registry = state.manager.registry().clone();
    match web::block(move || registry.revoke_api_token(&token_id)).await {
        Ok(Ok(())) => HttpResponse::Ok().json(serde_json::json!({"revoked": true})),
        Ok(Err(e)) => crate::error::error_response(e),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
