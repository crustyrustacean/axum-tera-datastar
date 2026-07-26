// src/main.rs

use axum_tera_datastar::AppState;
use axum_tera_datastar::Application;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // read host and port from environment variables
    dotenvy::dotenv()?;

    // construct the application state
    let app_state = AppState::default();

    // configure the app address
    let host = env::var("HOST")?;
    let port = env::var("PORT")?;
    let app_address = format!("{}:{}", host, port);

    // build the application, passing in the address and app state
    let app = Application::build(app_address, app_state).await?;

    // run the application
    app.run_until_stopped().await?;

    Ok(())
}
