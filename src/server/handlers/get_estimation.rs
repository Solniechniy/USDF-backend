use {
    axum::{extract::State, Json},
    tracing::instrument,
};

use super::utils::{calculate_usdf_amount, EstimationInput};
use crate::{error::AppError, state::AppState};

/// Handle get estimation requests
#[instrument(level = "info", skip(state))]
pub(crate) async fn get_estimation_handler(
    state: State<AppState>,
    Json(input): Json<EstimationInput>,
) -> Result<String, AppError> {
    let mut connection = state.redis.get_connection()?;
    let usdf_amount = calculate_usdf_amount(&mut connection, &input.token_address, input.amount)?;

    Ok(usdf_amount.to_string())
}
