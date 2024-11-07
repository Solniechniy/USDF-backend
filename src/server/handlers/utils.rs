use {
    crate::state::PriceData, redis::Commands, rlp::RlpStream, serde::{Deserialize, Serialize}, sha2::{Digest, Sha256}, tracing::instrument
};

use crate::error::AppError;

pub(crate) const USDF_COEFFICIENT: u128 = 30;
const BASE: u128 = 10;

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
    let value: String = redis_connection
        .get(token_address)
        .map_err(|_| AppError::invalid_request("Unknown token"))?;

    let price_data: PriceData = serde_json::from_str(&value)?;
    let price_u128: u128 = price_data.price.parse().expect("Failed to parse string to u128");
    let decimals_value = BASE.pow(price_data.decimals as u32);

    Ok(amount * price_u128 * USDF_COEFFICIENT / 100 / decimals_value)
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
