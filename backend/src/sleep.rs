use std::time::Duration;

// Conditionally import the correct sleep function based on the target
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;

#[cfg(target_arch = "wasm32")]
use wasmtimer::tokio::sleep;

/// Sleeps for 1 second, integrating with Tokio on native and the
/// browser/WASM runtime via wasmtimer.
pub async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}
