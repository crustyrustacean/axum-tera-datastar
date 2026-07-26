// src/app.rs

use crate::AppState;
use crate::routes::{get_index_page, post_new_item_ds};
use crate::shutdown_signal;
use crate::telemetry::MakeRequestUuid;
use axum::{
    Router,
    http::HeaderName,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    services::ServeDir,
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

pub struct Application {
    listener: TcpListener,
    router: Router,
}

impl Application {
    pub async fn build(addr: &str, app_state: AppState) -> anyhow::Result<Self> {
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
    
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .include_headers(true)
                .level(Level::INFO),
        )
        .on_response(DefaultOnResponse::new().include_headers(true));
    
    Router::new()
        .route("/", get(get_index_page))
        .route("/items", post(post_new_item_ds))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    X_REQUEST_ID.clone(),
                    MakeRequestUuid,
                ))
                .layer(trace_layer)
                .layer(PropagateRequestIdLayer::new(X_REQUEST_ID)),
        )
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
