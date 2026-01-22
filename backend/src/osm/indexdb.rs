use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;

// TODO: convert filename to a JavaScript identifier.
// In JavaScript, identifiers can contain Unicode letters, $, _, and digits (0-9),
// but may not start with a digit.
// https://developer.mozilla.org/en-US/docs/Glossary/Identifier
// 48.650/8.350/48.700/8.400/village
fn identifier(filename: &str) -> String {
    let mut ret = filename.to_string();
    ret = ret.replace(".", "_");
    ret = ret.replace("/", "-");
    format!("id-{}", ret)
}

const DATABASE: &str = "db";
const STORE: &str = "store";
//const TRANSACTION: &str = "store";

async fn opendb() -> Option<Database> {
    match Database::open(DATABASE)
        .with_version(1u8)
        .with_on_upgrade_needed(|event, db| {
            // Convert versions from floats to integers to allow using them in match expressions
            let old_version = event.old_version() as u64;
            let new_version = event.new_version().map(|v| v as u64);
            log::info!("old: {:?}, new: {:?}", old_version, new_version);
            match (old_version, new_version) {
                (0, Some(1)) => {
                    db.create_object_store(STORE)
                        .with_auto_increment(true)
                        .build()?;
                }
                (prev, Some(2)) => {
                    assert!(false);
                    if prev == 1 {
                        let _ = db.delete_object_store(STORE);
                    }
                    db.create_object_store("my_other_store").build()?;
                }
                _ => {}
            }

            Ok(())
        })
        .await
    {
        Ok(db) => Some(db),
        Err(e) => {
            log::info!("could not open db {:?}", e);
            None
        }
    }
}

async fn awrite(filename: &str, data: String) {
    let db = match opendb().await {
        Some(db) => db,
        None => {
            return;
        }
    };
    // Populate some data
    let transaction = db
        .transaction(STORE)
        .with_mode(TransactionMode::Readwrite)
        .build();

    match transaction {
        Ok(t) => {
            let store = t.object_store(STORE).unwrap();
            match store.put(data).with_key(identifier(filename)).await {
                Ok(s) => {
                    log::info!("write: {}", s);
                }
                Err(e) => {
                    log::info!("could not put data because {:?}", e);
                    return;
                }
            }
            match t.commit().await {
                Ok(()) => {
                    log::info!("commit ok");
                }
                Err(e) => {
                    // I get this error:
                    //
                    //   could not commit because DomException(InvalidStateError(DomException {
                    //   obj: Object { obj: JsValue(InvalidStateError: An attempt was made to
                    //   use an object that is not, or is no longer, usable.
                    //
                    // I think commit is not necessary. I just print the error and move on.
                    log::info!("could not commit because {:?}", e);
                }
            }
        }
        Err(e) => {
            log::info!("could not open transaction because {:?}", e);
        }
    }
}

pub async fn write(filename: &str, data: String) {
    let _ = awrite(filename, data).await;
}

async fn aread(filename: &str) -> Option<String> {
    let db = match opendb().await {
        Some(db) => db,
        None => {
            return None;
        }
    };
    let transaction = match db
        .transaction(STORE)
        .with_mode(TransactionMode::Readonly)
        .build()
    {
        Ok(d) => d,
        Err(e) => {
            log::error!("couldn not open transaction because {:?}", e);
            return None;
        }
    };

    let store = transaction.object_store(STORE).unwrap();
    let data = store.get(identifier(&filename)).await.unwrap();
    data
}

async fn ahit_cache(filename: &str) -> bool {
    let db = match opendb().await {
        Some(db) => db,
        None => {
            return false;
        }
    };
    let transaction = match db
        .transaction(STORE)
        .with_mode(TransactionMode::Readonly)
        .build()
    {
        Ok(d) => d,
        Err(e) => {
            log::error!("couldn not open transaction because {:?}", e);
            return false;
        }
    };

    let store = transaction.object_store(STORE).unwrap();
    let data = store.get_key(identifier(&filename)).await.unwrap();
    data.is_some()
}

pub async fn read(filename: &str) -> Option<String> {
    aread(filename).await
}

#[allow(dead_code)]
pub async fn hit_cache(filename: &str) -> bool {
    ahit_cache(filename).await
}
