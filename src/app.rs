// src/app.rs

use crate::AppState;
use crate::routes::{get_index_page, post_new_item_ds};
use crate::shutdown_signal;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub struct Application {
    listener: TcpListener,
    router: Router,
}

impl Application {
    pub async fn build(addr: String, app_state: AppState) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        let router = build_router(app_state);

        Ok(Self { listener, router })
    }

    pub fn port(&self) -> std::io::Result<u16> {
        Ok(self.listener.local_addr()?.port())
    }

    pub async fn run_until_stopped(self) -> std::io::Result<()> {
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_index_page))
        .route("/items", post(post_new_item_ds))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
