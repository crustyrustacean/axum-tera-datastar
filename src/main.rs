// src/main.rs

use axum::{
    Router,
    extract::{Form, State},
    response::Html,
    routing::get,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};
use tokio::signal;
use tower_http::services::ServeDir;

#[derive(Clone, Debug)]
struct AppState {
    templates: Tera,
    items: Arc<Mutex<Vec<String>>>,
}

#[derive(Deserialize, Serialize)]
struct NewItem {
    item: String,
}

#[debug_handler]
async fn hello_world(
    State(state): State<AppState>,
    Form(new_item): Form<NewItem>,
) -> Html<String> {
    state.items.lock().unwrap().push(new_item.item);
    let mut context = Context::new();
    context.insert("items", &state.items.lock().unwrap());
    Html(state.templates.render("index.html", &context).unwrap())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut tera = Tera::default();
    tera.load_from_glob("templates/**/*.html")?;

    let app_state = AppState {
        templates: tera,
        items: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/", get(hello_world))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
