use {
    axum::{extract::State, Json},
    serde_json::{json, Value},
    tracing::instrument,
};

use super::utils::{calculate_usdf_amount, create_asset_msg, SignatureInput};
use crate::{error::AppError, state::AppState};

/// Handle get signature requests
#[instrument(level = "info", skip(state))]
pub(crate) async fn get_signature_handler(
    mut state: State<AppState>,
    Json(input): Json<SignatureInput>,
) -> Result<Json<Value>, AppError> {
    let mut connection = state.redis.get_connection()?;

    let nonce = state.generate_nonce(&mut connection).await?;
    let usdf_amount = calculate_usdf_amount(&mut connection, &input.token_address, input.amount)?;

    let message = create_asset_msg(&input, usdf_amount, nonce);
    let signature = state.sign(&message);

    let res = json!({
            "nonce" : nonce.to_string(),
            "usdf_amount" : usdf_amount.to_string(),
            "signature" : hex::encode(signature.to_bytes())
        }
    );

    Ok(Json(res))
}
