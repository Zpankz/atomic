//! Database resolution extractor for actix-web.
//!
//! `Db` is a `FromRequest` extractor that resolves the correct `AtomicCore`
//! from the request (via `X-Atomic-Database` header, `?db=` param, or active db).

use crate::state::AppState;
use actix_web::{web, FromRequest, HttpRequest};
use atomic_core::AtomicCore;
use std::future::{ready, Ready};

/// Extractor that resolves the correct AtomicCore for the current request.
pub struct Db(pub AtomicCore);

impl FromRequest for Db {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let state = match req.app_data::<web::Data<AppState>>() {
            Some(s) => s,
            None => {
                return ready(Err(actix_web::error::ErrorInternalServerError(
                    "AppState not configured",
                )));
            }
        };

        match state.resolve_core(req) {
            Ok(core) => ready(Ok(Db(core))),
            Err(e) => ready(Err(actix_web::error::ErrorBadRequest(format!(
                "Database not found: {}",
                e
            )))),
        }
    }
}
