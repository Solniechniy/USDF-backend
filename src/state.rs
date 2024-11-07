use {
    anyhow::{anyhow, Result}, ed25519_dalek::{ed25519::signature::SignerMut, Signature, SigningKey}, redis::Commands, secrecy::ExposeSecret, serde::{Deserialize, Serialize}, std::sync::Arc, tokio::sync::RwLock, tracing::info
};

use crate::configuration::AppConfig;

pub const LAST_NONCE_KEY: &str = "last_nonce";

#[derive(Clone)]
pub(crate) struct AppState {
    pub redis: redis::Client,
    signing_key: SigningKey,
    pub last_nonce: Arc<RwLock<u64>>,
}

#[derive(Serialize, Deserialize)]
pub struct PriceData {
    pub price: String,
    pub decimals: u8,
}

impl AppState {
    pub fn init(configuration: &AppConfig) -> Result<Self> {
        let signing_key: SigningKey = load_signing_key(configuration)
            .map_err(|e| anyhow!("Failed to load signing keys: {}", e))?;
        let redis = open_redis(configuration)
            .map_err(|e| anyhow!("Invalid redis configurations: {}", e))?;
        let mut connection = redis
            .get_connection()
            .map_err(|e| anyhow!("Failed to open redis connection: {}", e))?;

        let last_nonce = read_nonce(&mut connection).unwrap_or(0);

        Ok(Self {
            redis,
            signing_key,
            last_nonce: Arc::new(RwLock::new(last_nonce)),
        })
    }

    /// Generates new nonce
    pub(crate) async fn generate_nonce(
        &mut self,
        connection: &mut redis::Connection,
    ) -> Result<u64> {
        let mut max_nonce = self.last_nonce.write().await;

        *max_nonce = max_nonce
            .checked_add(1)
            .ok_or(anyhow!("Max nonce overflow"))?;

        save_nonce(connection, *max_nonce).map_err(|_| anyhow!("Failed to save persistance"))?;
        info!(nonce = *max_nonce, "Nonce updated");

        Ok(*max_nonce)
    }

    pub(crate) fn sign(&mut self, msg: &[u8]) -> Signature {
        self.signing_key.sign(msg)
    }
}

fn load_signing_key(configuration: &AppConfig) -> Result<SigningKey> {
    let keypair_bytes = bs58::decode(configuration.signing_key.expose_secret()).into_vec()?;
    let keypair: [u8; 64] = keypair_bytes
        .try_into()
        .map_err(|_| anyhow!("Failed to decode signing key"))?;

    let signing_key = SigningKey::from_keypair_bytes(&keypair)?;

    Ok(signing_key)
}

pub fn open_redis(configuration: &AppConfig) -> Result<redis::Client> {
    let redis = redis::Client::open(configuration.redis_uri.expose_secret())
        .map_err(|_| anyhow!("Invalid redis configuration"))?;

    Ok(redis)
}

fn save_nonce(connection: &mut redis::Connection, nonce: u64) -> Result<()> {
    connection.set(LAST_NONCE_KEY, nonce.to_string())?;

    Ok(())
}

fn read_nonce(connection: &mut redis::Connection) -> Result<u64> {
    let nonce: u64 = connection.get(LAST_NONCE_KEY)?;

    Ok(nonce)
}
