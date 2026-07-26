// src/main.rs

use axum::{
    Router,
    extract::State,
    response::{
        Html,
        sse::{Event, Sse},
    },
    routing::{get, post},
};
use axum_macros::debug_handler;
use datastar::{axum::ReadSignals, prelude::*};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};
use tokio::signal;
use tower_http::services::ServeDir;

#[derive(Clone, Debug)]
struct AppState {
    templates: Tera,
    items: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Deserialize, Serialize)]
struct NewItem {
    item: String,
}

#[debug_handler]
async fn get_index_page(State(state): State<AppState>) -> Html<String> {
    let items = state.items.lock().unwrap();
    let mut context = Context::new();
    context.insert("items", &*items);

    Html(state.templates.render("index.html", &context).unwrap())
}

#[debug_handler]
async fn post_new_item_ds(
    State(state): State<AppState>,
    ReadSignals(new_item): ReadSignals<NewItem>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mut items = state.items.lock().unwrap();
    items.push(new_item.item.clone());

    let patch = PatchElements::new(format!("<li>{}</li>", new_item.item))
        .selector("#item-list")
        .mode(ElementPatchMode::Append);

    Sse::new(tokio_stream::once(Ok(patch.write_as_axum_sse_event())))
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
        .route("/", get(get_index_page))
        .route("/items", post(post_new_item_ds))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
