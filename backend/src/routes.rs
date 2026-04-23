use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::AppState;
use crate::handlers::*;
use crate::middleware::{rate_limit, set_user_cookie};

pub fn create_routes(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        .route("/health", get(health))
        .route("/links", post(create_short_link).get(get_user_links))
        .route("/links/:code/stats", get(get_link_stats))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit,
        ))
        .route_layer(middleware::from_fn(set_user_cookie))
        .with_state(state.clone());

    let redirect_routes = Router::new()
        .route("/:code", get(redirect_short_link))
        .with_state(state.clone());

    Router::new()
        .nest("/api", api_routes)
        .merge(redirect_routes)
}
