use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;
use super::handlers::create_files_routes;

pub fn create_files_app(state: Arc<RwLock<AppState>>) -> Router {
    create_files_routes(state)
}
