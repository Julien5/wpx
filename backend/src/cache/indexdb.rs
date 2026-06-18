use indexed_db_futures::prelude::*;
use indexed_db_futures::IdbDatabase;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::DomException;

use crate::cache::Location;

/// Represents the distinct databases in your application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Database {
    UserData,
    OsmCache,
}

impl Database {
    pub fn from_location(b: &Location) -> Self {
        match b {
            Location::UserData => Self::UserData,
            Location::OsmCache => Self::OsmCache,
        }
    }
    fn name_version(&self) -> (String, u32) {
        let name = match self {
            Database::UserData => format!("{}", "UserData"),
            Database::OsmCache => format!("{}", "OsmCache"),
        };
        let version = match self {
            Database::UserData => 1,
            Database::OsmCache => 2,
        };
        (name, version)
    }

    /// Every database needs at least one object store to hold key-value data.
    /// We use a standard name across all databases for simplicity.
    fn store_name(&self) -> String {
        format!("{}", "store")
    }
}

// WASM is strictly single-threaded. We use a thread-local RefCell containing a HashMap
// to cache our open database connections. The `Rc` allows us to cheaply clone the connection
// without worrying about lifetime issues.
thread_local! {
    static DATABASES: RefCell<HashMap<Database, Rc<IdbDatabase>>> = RefCell::new(HashMap::new());
}

async fn get_db(database: &Database) -> Result<Rc<IdbDatabase>, JsValue> {
    // 1. Check cache
    let cached_db = DATABASES.with(|dbs| dbs.borrow().get(&database).cloned());
    if let Some(db) = cached_db {
        return Ok(db);
    }

    // 2. Fetch version from your enum
    let (name, version) = database.name_version();

    // 3. Initiate open request
    let mut db_req =
        IdbDatabase::open_u32(&name, version).map_err(|e: DomException| JsValue::from(e))?;

    let local_database = database.clone();
    // 4. Set the upgrade handler
    // We clone the enum variant to move it into the closure safely
    db_req.set_on_upgrade_needed(Some(
        move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            let db = evt.db();
            let store = local_database.store_name();

            // This logic runs whenever the requested version is higher than the existing version
            if !db.object_store_names().any(|n| n == store) {
                db.create_object_store(&store)
                    .map_err(|e: DomException| JsValue::from(e))?;
            }

            // FUTURE-PROOFING:
            // If you ever need complex migrations (e.g., version 1 -> 2),
            // you can check evt.old_version() here:
            // let old = evt.old_version();
            // if old < 2 { /* perform migration logic */ }

            Ok(())
        },
    ));

    // 5. Await the result
    let db: IdbDatabase = db_req.await.map_err(|e: DomException| JsValue::from(e))?;
    let db = Rc::new(db);

    // 6. Cache
    DATABASES.with(|dbs| {
        dbs.borrow_mut().insert(database.clone(), db.clone());
    });

    Ok(db)
}

/// Writes a string payload to the specified database under the given key.
pub async fn write(database: &Database, key: &str, data: String) -> Result<(), JsValue> {
    log::trace!("db write: {}", key);
    let db = get_db(database).await?;
    let store = database.store_name();

    // Open a read-write transaction on the specific database
    let tx = db.transaction_on_one_with_mode(&store, IdbTransactionMode::Readwrite)?;
    let object_store = tx.object_store(&store)?;

    // IndexedDB keys and values must be JsValues
    object_store
        .put_key_val(&JsValue::from_str(key), &JsValue::from_str(&data))?
        .await?;

    Ok(())
}

/// Reads a string payload from the specified database by its key.
pub async fn read(database: &Database, key: &str) -> Result<String, JsValue> {
    log::trace!("db read: {}", key);
    let db = get_db(database).await?;
    /*{
        Ok(d) => d,
        Err(e) => {
            return Err(e);
        }
    };*/
    let store = database.store_name();

    // Open a read-only transaction (faster than read-write)
    let tx = db.transaction_on_one_with_mode(&store, IdbTransactionMode::Readonly)?;
    let object_store = tx.object_store(&store)?;

    let result = object_store.get(&JsValue::from_str(key))?.await?;
    let ret = match result {
        Some(jsvalue) => {
            debug_assert!(jsvalue.is_string());
            Ok(jsvalue.as_string().unwrap())
        }
        None => Err(JsValue::from_str("bad")),
    };
    ret
}
