// src/get.rs

use crate::AppState;
use axum::{extract::State, response::Html};
use axum_macros::debug_handler;
use tera::Context;

#[debug_handler]
pub async fn get_index_page(State(state): State<AppState>) -> Html<String> {
    let items = state.items.lock().unwrap();
    let mut context = Context::new();
    context.insert("items", &*items);

    Html(state.templates.render("index.html", &context).unwrap())
}
