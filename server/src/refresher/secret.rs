use std::time::Duration;

use log::info;
use tokio::time::Instant;

use crate::consensus::handler::ConsensusHandler;
use crate::domain::error::SecretServerError;

async fn refresh_secret(consensus_handler: ConsensusHandler) -> Result<(), SecretServerError> {
    if consensus_handler.is_begin_refresh() {
        info!("Secrets are being refreshed, skipping this refresh");
        return Ok(());
    }
    info!("Refreshing all secrets share with new random polynomial coefficients");
    consensus_handler.start_refresh().await?;
    consensus_handler.refresh_secrets().await?;
    consensus_handler.finish_refresh().await
}

pub async fn run(interval_secs: u64, consensus_handler: ConsensusHandler) {
    let mut start_time = Instant::now();

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
