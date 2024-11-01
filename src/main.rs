use {
    anyhow::Result, application::Application, configuration::AppConfig,
    observability::init_tracing, std::time::Duration, tokio::time::sleep, tracing::debug,
};

pub(crate) mod application;
pub(crate) mod configuration;
pub(crate) mod error;
pub(crate) mod observability;
pub(crate) mod server;
pub(crate) mod state;

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = AppConfig::load()?;

    // Sleep for std initialization
    sleep(Duration::from_millis(100)).await;

    init_tracing(&configuration.log_level, configuration.is_json_logging)?;
    debug!(version = env!("CARGO_PKG_VERSION"), "Application started");

    let mut app = Application::from_configuration(configuration).await?;

    app.run_usdf_server().await
}
