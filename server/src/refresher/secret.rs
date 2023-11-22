use std::time::Duration;

use log::info;
use tokio::time::Instant;

use crate::consensus::handler::ConsensusHandler;
use crate::domain::error::SecretServerError;

/// Asynchronously refreshes secrets.
///
/// The function takes a `ConsensusHandler` as an argument and attempts to refresh the secrets.
/// If `is_begin_refresh` returns `true`, the function skips the refresh process and returns early.
/// Otherwise, it sends a start refresh message and waits for the response.
/// If the response is successful, it proceeds to refresh the secrets and then finishes the refresh process.
///
/// # Arguments
///
/// * `consensus_handler` - The consensus handler used for secret refreshing.
///
/// # Returns
///
/// Returns `Ok(())` if the secrets are refreshed successfully, otherwise returns a `SecretServerError`.
async fn refresh_secret(consensus_handler: ConsensusHandler) -> Result<(), SecretServerError> {
    if consensus_handler.is_begin_refresh() {
        info!("Secrets are being refreshed, skipping this refresh");
        return Ok(());
    }
    info!("Refreshing all secrets share with new random polynomial coefficients");
    let start = consensus_handler.start_refresh().await;
    if let Ok(_) = start {
        info!("Start refresh message sent successfully");
        consensus_handler.refresh_secrets().await?;
        consensus_handler.finish_refresh().await
    } else {
        Err(SecretServerError::RefreshError)
    }
}

/// Runs the secret refresher task with the specified interval.
///
/// The function initializes a timer to keep track of the start time.
/// It then enters an infinite loop where it calculates the next execution time based on the interval.
/// It sleeps until the next execution time is reached, and then calls the `refresh_secret` function.
/// If the function succeeds, it logs a success message, otherwise it logs an error message.
/// Finally, it updates the start time for the next iteration.
///
/// # Arguments
///
/// * `interval_secs` - The interval in seconds between each secret refresh task.
/// * `consensus_handler` - The consensus handler used for secret refreshing.
pub async fn run(interval_secs: u64, consensus_handler: ConsensusHandler) {
    let mut start_time = Instant::now() + Duration::from_secs(10);

    info!(
        "Starting secret refresher task with interval {} seconds",
        interval_secs
    );

    loop {
        // Calculate the next time the task should be executed based on the interval
        let next_execution_time = start_time + Duration::from_secs(interval_secs);

        // Sleep until the next execution time
        tokio::time::sleep_until(next_execution_time).await;

        // Perform your task here
        let result = refresh_secret(consensus_handler.clone()).await;
        match result {
            Ok(_) => info!("Secrets refreshed successfully"),
            Err(e) => info!("Error refreshing secrets: {}", e),
        }

        // Update the start time for the next iteration
        start_time = next_execution_time;
    }
}
