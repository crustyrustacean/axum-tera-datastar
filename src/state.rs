// src/state.rs

use std::sync::{Arc, Mutex};
use tera::Tera;

#[derive(Clone, Debug)]
pub struct AppState {
    pub templates: Tera,
    pub items: Arc<Mutex<Vec<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        let mut tera = Tera::default();
        tera.load_from_glob("templates/**/*.html")
            .expect("Unable to load the Tera templates.");

        Self {
            templates: tera,
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
