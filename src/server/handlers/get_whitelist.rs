use {
    axum::{extract::State, Json},
    redis::Commands,
    serde_json::{json, Value},
    tracing::instrument,
};

use super::utils::USDF_COEFFICIENT;
use crate::{error::AppError, state::AppState};

/// Handle get whitelist requests
#[instrument(level = "info", skip(state))]
pub(crate) async fn get_whitelist_handler(
    state: State<AppState>,
) -> Result<Json<Vec<Value>>, AppError> {
    let mut redis_connection = state.redis.get_connection()?;
    let mut res = vec![];
    let keys: Vec<String> = redis_connection.keys("*")?;

    for token in keys {
        let price: String = redis_connection.get(&token)?;
        let info = json!({
            "token" : token,
            "price" : price,
            "coefficient" : USDF_COEFFICIENT
        });
        res.push(info);
    }

    Ok(Json(res))
}
