use {
    anyhow::{anyhow, Result},
    axum::{
        http::HeaderValue,
        routing::{get, post},
        Router, Server,
    },
    tokio::{signal, sync::oneshot},
    tower_http::cors::{AllowOrigin, CorsLayer},
    tracing::info,
};

use super::handlers::{
    get_estimation_handler, get_signature_handler, get_whitelist_handler, health_handler,
};

use crate::application::Application;

impl Application {
    async fn create_router(&self) -> Result<Router> {
        let header = "http://localhost".parse::<HeaderValue>()?;
        let cors = CorsLayer::new().allow_origin(AllowOrigin::exact(header));

        let router = Router::new()
            .route("/health", get(health_handler))
            .route("/get_whitelist", get(get_whitelist_handler))
            .route("/get_signature", post(get_signature_handler))
            .route("/get_estimation", post(get_estimation_handler))
            .with_state(self.state.clone())
            .layer(cors);

        Ok(router)
    }

    pub async fn run_server(&mut self) -> Result<()> {
        tracing::info!(address = ?self.socket, "Start server");

        let (tx, rx) = oneshot::channel::<()>();
        self.shutdown_sender = Some(tx);

        let server = Server::bind(&self.socket)
            .serve(self.create_router().await?.into_make_service())
            .with_graceful_shutdown(async {
                rx.await.ok();
                info!("Signal received, starting graceful shutdown");
            });

        // Store the server task's handle
        self.server_handle = Some(tokio::spawn(
            async move { server.await.map_err(|e| anyhow!(e)) },
        ));

        Ok(())
    }

    pub(crate) async fn handle_shutdown_signal(&mut self) -> Result<()> {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        let interrupt = async {
            signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        let quit = async {
            signal::unix::signal(signal::unix::SignalKind::quit())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        tokio::select! {
                _ = ctrl_c => info!("Ctrl-c received!"),
                _ = terminate => info!("Terminate received!"),
                _ = interrupt => info!("Interrupt received!"),
                _ = quit => info!("Quit received!"),
        };

        self.shutdown().await
    }
}
