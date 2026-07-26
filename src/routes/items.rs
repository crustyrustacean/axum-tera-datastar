// src/post.rs

use crate::AppState;
use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum_macros::debug_handler;
use datastar::{axum::ReadSignals, prelude::*};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

#[derive(Clone, Deserialize, Serialize)]
pub struct NewItem {
    item: String,
}

#[debug_handler]
pub async fn post_new_item_ds(
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
