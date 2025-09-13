#[cfg(not(target_family = "wasm"))]
use tokio::time::*;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio::*;

use std::time::Duration;

#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;

async fn processdb() -> indexed_db_futures::OpenDbResult<()> {
    let db = Database::open("my_db")
        .with_version(2u8)
        .with_on_upgrade_needed(|event, db| {
            // Convert versions from floats to integers to allow using them in match expressions
            let old_version = event.old_version() as u64;
            let new_version = event.new_version().map(|v| v as u64);

            match (old_version, new_version) {
                (0, Some(1)) => {
                    db.create_object_store("my_store")
                        .with_auto_increment(true)
                        .build()?;
                }
                (prev, Some(2)) => {
                    if prev == 1 {
                        let _ = db.delete_object_store("my_store");
                    }

                    db.create_object_store("my_other_store").build()?;
                }
                _ => {}
            }

            Ok(())
        })
        .await?;

    // Populate some data
    let transaction = db
        .transaction("my_other_store")
        .with_mode(TransactionMode::Readwrite)
        .build()?;

    let store = transaction.object_store("my_other_store")?;

    store
        .put("a primitive value that doesn't need serde")
        .with_key("my_key")
        .await?;

    // Unlike JS, transactions ROLL BACK INSTEAD OF COMMITTING BY DEFAULT
    transaction.commit().await?;

    // Read some data
    let transaction = db.transaction("my_other_store").build()?;
    let store = transaction.object_store("my_other_store")?;

    Ok(())
}

pub async fn process(count: &i32) -> i32 {
    sleep(Duration::from_millis(1000)).await;
    processdb().await;
    count + 1
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
