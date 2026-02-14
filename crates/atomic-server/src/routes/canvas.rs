//! Canvas position routes

use crate::error::ok_or_error;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use atomic_core::AtomPosition;

pub async fn get_positions(state: web::Data<AppState>) -> HttpResponse {
    ok_or_error(state.core.get_atom_positions())
}

pub async fn save_positions(
    state: web::Data<AppState>,
    body: web::Json<Vec<AtomPosition>>,
) -> HttpResponse {
    let positions = body.into_inner();
    match state.core.save_atom_positions(&positions) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"status": "ok"})),
        Err(e) => crate::error::error_response(e),
    }
}

pub async fn get_atoms_with_embeddings(state: web::Data<AppState>) -> HttpResponse {
    ok_or_error(state.core.get_atoms_with_embeddings())
}
