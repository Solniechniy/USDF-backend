use {
    redis::Commands,
    rlp::RlpStream,
    serde::{Deserialize, Serialize},
    sha2::{Digest, Sha256},
    tracing::instrument,
};

use crate::error::AppError;

pub(crate) const USDF_COEFFICIENT: u128 = 30;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SignatureInput {
    pub user_address: String,
    pub token_address: String,
    pub amount: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct EstimationInput {
    pub token_address: String,
    pub amount: u128,
}

/// Create asset message
#[instrument(level = "debug", skip(redis_connection))]
pub(crate) fn calculate_usdf_amount(
    redis_connection: &mut redis::Connection,
    token_address: &str,
    amount: u128,
) -> Result<u128, AppError> {
    let price: u128 = redis_connection
        .get(token_address)
        .map_err(|_| AppError::invalid_request("Unknown token"))?;

    Ok(amount * price * USDF_COEFFICIENT / 100)
}

/// Create message asset msg
#[instrument(level = "debug")]
pub(crate) fn create_asset_msg(input: &SignatureInput, usdf_amount: u128, nonce: u64) -> Vec<u8> {
    let mut stream = RlpStream::new_list(5);
    let mut hasher = Sha256::new();

    stream.append(&nonce.to_be_bytes().as_ref());
    stream.append(&input.token_address.as_bytes());
    stream.append(&input.amount.to_be_bytes().as_ref());
    stream.append(&usdf_amount.to_be_bytes().as_ref());
    stream.append(&input.user_address.as_bytes());

    let message = stream.out().to_vec();
    hasher.update(message);

    hasher.finalize().to_vec()
}
