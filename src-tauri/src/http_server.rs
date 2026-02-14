use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use atomic_core::AtomicCore;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp_actix_web::transport::StreamableHttpService;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;

use crate::event_bridge::embedding_event_callback;
use crate::mcp::AtomicMcpServer;
use crate::models::CreateAtomRequest;

pub struct AppState {
    pub core: AtomicCore,
    pub app_handle: tauri::AppHandle,
}

// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Create atom endpoint
async fn create_atom(
    state: web::Data<AppState>,
    payload: web::Json<CreateAtomRequest>,
) -> impl Responder {
    let request: atomic_core::CreateAtomRequest = payload.into_inner().into();
    let on_event = embedding_event_callback(state.app_handle.clone());

    match state.core.create_atom(request, on_event) {
        Ok(atom) => {
            // Emit event to frontend to trigger immediate UI refresh
            state.app_handle.emit("atom-created", &atom).ok();
            HttpResponse::Ok().json(atom)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn start_server(
    core: AtomicCore,
    app_handle: tauri::AppHandle,
) -> std::io::Result<()> {
    let port = 44380; // Uncommon port, unlikely to conflict

    let app_state = web::Data::new(AppState {
        core: core.clone(),
        app_handle: app_handle.clone(),
    });

    // Create MCP service - must be created outside HttpServer::new for worker sharing
    let mcp_core = core.clone();
    let mcp_handle = app_handle.clone();

    let mcp_service = StreamableHttpService::builder()
        .service_factory(Arc::new(move || {
            Ok(AtomicMcpServer::new(mcp_core.clone(), mcp_handle.clone()))
        }))
        .session_manager(Arc::new(LocalSessionManager::default()))
        .stateful_mode(false) // Stateless since we share DB state
        .sse_keep_alive(Duration::from_secs(30))
        .build();

    println!("Starting HTTP server on http://127.0.0.1:{}", port);
    println!("MCP endpoint available at http://127.0.0.1:{}/mcp", port);

    HttpServer::new(move || {
        // Allow extension to make requests
        let cors = Cors::permissive(); // Localhost only, so permissive is fine

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            // Existing routes
            .route("/health", web::get().to(health))
            .route("/atoms", web::post().to(create_atom))
            // MCP routes
            .service(web::scope("/mcp").service(mcp_service.clone().scope()))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
