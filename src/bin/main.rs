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
use axum_tera_datastar::shutdown_signal;
use datastar::{axum::ReadSignals, prelude::*};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::env;
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};
use tokio::net::TcpListener;
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
        .mode(ElementPatchMode::Append)
        .write_as_axum_sse_event();

    let clear = PatchSignals::new(r#"{"item":""}"#).write_as_axum_sse_event();

    Sse::new(tokio_stream::iter(vec![Ok(patch), Ok(clear)]))
}

struct Application {
    listener: TcpListener,
    app: Router,
}

impl Application {
    async fn build(addr: &str, app_state: AppState) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        let app = Router::new()
            .route("/", get(get_index_page))
            .route("/items", post(post_new_item_ds))
            .nest_service("/static", ServeDir::new("static"))
            .with_state(app_state);

        Ok(Self { listener, app })
    }

    #[allow(dead_code)]
    fn port(&self) -> std::io::Result<u16> {
        Ok(self.listener.local_addr()?.port())
    }

    async fn run_until_stopped(self) -> std::io::Result<()> {
        axum::serve(self.listener, self.app)
            .with_graceful_shutdown(shutdown_signal())
            .await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // read host and port from environment variables
    dotenvy::dotenv()?;

    // load the templates
    let mut tera = Tera::default();
    tera.load_from_glob("templates/**/*.html")?;

    // build the app state, includes templates and in-memory storage
    let app_state = AppState {
        templates: tera,
        items: Arc::new(Mutex::new(Vec::new())),
    };

    // configure the app address
    let host = env::var("HOST")?;
    let port = env::var("PORT")?;
    let address = format!("{}:{}", host, port);

    // build the application, passing in the address and app state
    let application = Application::build(&address, app_state).await?;

    // run the application
    application.run_until_stopped().await?;

    Ok(())
}
