// src/main.rs

use axum::{Router, extract::State, response::Html, routing::get};
use axum_macros::debug_handler;
use tera::{Context, Tera};
use tokio::signal;

#[debug_handler]
async fn hello_world(State(tera): State<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("message", "Axum, Tera, and Datastar FTW");
    Html(tera.render("index.html", &context).unwrap())
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

    let app = Router::new().route("/", get(hello_world)).with_state(tera);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
