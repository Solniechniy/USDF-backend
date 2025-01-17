use {
    anyhow::{anyhow, Result},
    redis::Commands,
    std::net::SocketAddr,
    tokio::{net::TcpListener, sync::oneshot, task::JoinHandle},
    tracing::info,
};

use crate::{configuration::AppConfig, state::{AppState, PriceData}};

pub struct Application {
    pub socket: SocketAddr,
    pub state: AppState,
    pub shutdown_sender: Option<oneshot::Sender<()>>,
    pub server_handle: Option<JoinHandle<Result<()>>>,
}

impl Application {
    pub async fn from_configuration(configuration: AppConfig) -> Result<Self> {
        let socket = TcpListener::bind(configuration.listener.to_owned())
            .await?
            .local_addr()
            .map_err(|e| anyhow!("Failed start listener: {}", e))?;

        let state = AppState::init(&configuration)?;

        info!(address = configuration.listener, "Application initialized");

        Ok(Self {
            socket,
            state,
            shutdown_sender: None,
            server_handle: None,
        })
    }

    pub(crate) async fn run_usdf_server(&mut self) -> Result<()> {
        self.run_server().await?;
        self.run_price_fetching().await?;
        self.handle_shutdown_signal().await
    }

    async fn run_price_fetching(&self) -> Result<()> {
        // WARN: HARDCODED TEST VERSION!
        // Further price + decimals will be fetched from DexTools
        let mut connection = self.state.redis.get_connection()?;
        // WARN: PRICE DECIMAL 18

        let price_data_usmeme = PriceData {
            price: "68420000000000".to_string(),
            decimals: 8,
        };

         let price_data_dd = PriceData {
            price: "800000000000000".to_string(),
            decimals: 8,
        };

         let price_data_testnet = PriceData {
            price: "800000000000000".to_string(),
            decimals: 18
        };


        connection.set("usmeme.tg", serde_json::to_string(&price_data_usmeme)?)?;
        connection.set("dd.tg", serde_json::to_string(&price_data_dd)?)?;
        connection.set("poken.sergei24.testnet", serde_json::to_string(&price_data_testnet)?)?;
        Ok(())
    }

    pub(crate) async fn shutdown(&mut self) -> Result<()> {
        // Send shutdown signal
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }

        // Wait for the server task to complete
        if let Some(handle) = self.server_handle.take() {
            handle.await??;
        }

        info!("Application shutdown");
        Ok(())
    }
}
