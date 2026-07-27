// src/main.rs

use axum_tera_datastar::AppState;
use axum_tera_datastar::Application;
use axum_tera_datastar::get_configuration;
use axum_tera_datastar::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber("axum-tera-datastar".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // construct the application state
    let app_state = AppState::default();

    // read the application settings
    let settings = get_configuration()?;
    let app_address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );

    // build the application, passing in the address and app state
    let app = Application::build(&app_address, app_state).await?;

    // run the application
    app.run_until_stopped().await?;

    Ok(())
}
