use {
    axum::{extract::State, Json},
    redis::Commands,
    serde_json::{json, Value},
    tracing::instrument,
};

use super::utils::USDF_COEFFICIENT;
use crate::{error::AppError, state::{AppState, PriceData, LAST_NONCE_KEY}};

/// Handle get whitelist requests
#[instrument(level = "info", skip(state))]
pub(crate) async fn get_whitelist_handler(
    state: State<AppState>,
) -> Result<Json<Vec<Value>>, AppError> {
    let mut redis_connection = state.redis.get_connection()?;
    let mut res = vec![];
    let keys: Vec<String> = redis_connection.keys("*")?;

    for token in keys.into_iter().filter(|key| key != LAST_NONCE_KEY) {
        let value: String = redis_connection.get(&token)?;
        let serialized_data: PriceData = serde_json::from_str(&value)?;
        let info = json!({
            "token" : token,
            "price" : serialized_data.price,
            "coefficient" : USDF_COEFFICIENT,
            "decimals": serialized_data.decimals
        });
        res.push(info);
    }

    Ok(Json(res))
}
