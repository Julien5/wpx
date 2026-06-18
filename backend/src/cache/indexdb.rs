use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;
use std::cell::OnceCell;

const DATABASE: &str = "db";

/// Fallback store for files with no leading directory, or an unknown dirname.
const DEFAULT_STORE: &str = "default";

/// All object stores that exist in the database.
/// - Add a new entry here when a new top-level directory is needed.
/// - Then bump the DB version in `get_db` and add a migration branch.
const KNOWN_STORES: &[&str] = &["osm", "tiles", "cache"];

// Bump this when KNOWN_STORES changes.
const DB_VERSION: u8 = 2;

// ---------------------------------------------------------------------------
// Filename decomposition
// ---------------------------------------------------------------------------

/// Splits a filename into (store_name, key).
///
/// The dirname is looked up in `KNOWN_STORES`:
///   "osm/1/E009-N060"  -> ("osm",     "1/E009-N060")   known store
///   "foo/bar"          -> ("default",  "foo/bar")       unknown dirname -> fallback
///   "bare_file"        -> ("default",  "bare_file")     no slash -> fallback
///
/// When falling back to DEFAULT_STORE the full filename is used as the key
/// to avoid collisions between e.g. "foo/x" and "bar/x" both becoming "x".
fn split_path(filename: &str) -> (&str, &str) {
    if let Some(pos) = filename.find('/') {
        let dir = &filename[..pos];
        if KNOWN_STORES.contains(&dir) {
            return (dir, &filename[pos + 1..]);
        }
    }
    (DEFAULT_STORE, filename)
}

// ---------------------------------------------------------------------------
// Cached database handle
// ---------------------------------------------------------------------------

// WASM is single-threaded, so a thread_local OnceCell is safe and avoids
// the Send + Sync requirements that a static would impose on Database.
thread_local! {
    static DB: OnceCell<Database> = OnceCell::new();
}

use thiserror::Error;
#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("OpenFailed")]
    OpenFailed,
}

/// Returns a reference to the cached Database, opening it on the first call.
///
/// Because WASM is single-threaded we borrow the OnceCell value inside the
/// thread_local for the duration of each async operation.  The database
/// handle stays alive for the lifetime of the page.
///
/// Store creation: IndexedDB requires all object stores to be declared
/// during an `upgradeneeded` event.  Since we derive store names from
/// directory names at runtime we can't know them all upfront.  The
/// standard workaround is to open the database WITHOUT specifying stores
/// in `upgradeneeded` and instead create each store lazily by bumping the
/// version when a new dirname appears.  However, that requires tracking
/// which stores exist and re-opening with a new version -- complex and slow.
///
/// Simpler alternative chosen here: KNOWN_STORES defines the fixed set of
/// stores created at version 1.  Unknown dirnames fall through to
/// DEFAULT_STORE.  To add a new store: push to KNOWN_STORES, bump the
/// version constant below, and add a migration branch.
///
/// If you truly need fully dynamic stores at runtime, replace this function
/// with one that tracks known stores in localStorage and re-opens with a
/// bumped version when a new dirname is encountered.
async fn get_db() -> Result<&'static Database, Error> {
    // Check if already initialised (fast path -- no async work).
    let cached = DB.with(|cell| {
        // SAFETY: we extend the lifetime of the reference to 'static.
        // This is sound because the OnceCell lives in a thread_local that
        // exists for the entire page lifetime, and WASM is single-threaded
        // so there is no concurrent access.
        cell.get().map(|db| unsafe { &*(db as *const Database) })
    });

    if let Some(db) = cached {
        return Ok(db);
    }

    // First call: open the database.
    let db = match Database::open(DATABASE)
        .with_version(1u8)
        .with_on_upgrade_needed(|event, db| {
            let old_version = event.old_version() as u64;
            let new_version = event.new_version().map(|v| v as u64);
            log::info!("IDB upgrade: {:?} -> {:?}", old_version, new_version);

            match (old_version, new_version) {
                (0, Some(1)) => {
                    // Fallback store for bare filenames and unknown dirnames.
                    db.create_object_store(DEFAULT_STORE)
                        .with_auto_increment(false)
                        .build()?;

                    // One store per entry in KNOWN_STORES.
                    for store_name in KNOWN_STORES {
                        db.create_object_store(store_name)
                            .with_auto_increment(false)
                            .build()?;
                    }
                }
                // Example of how to add a new store in a future version:
                //
                // (1, Some(2)) => {
                //     db.create_object_store("new_store")
                //         .with_auto_increment(false)
                //         .build()?;
                // }
                _ => {}
            }
            Ok(())
        })
        .await
    {
        Ok(db) => db,
        Err(e) => {
            log::error!("could not open db: {}", e);
            return Err(Error::OpenFailed);
        }
    };

    // Store in the thread_local and return a 'static reference.
    DB.with(|cell| {
        let _ = cell.set(db); // ignore error if another task raced us
        Ok(unsafe { &*(cell.get().unwrap() as *const Database) })
    })
}

// ---------------------------------------------------------------------------
// Write
// ---------------------------------------------------------------------------

async fn awrite(filename: &str, data: String) -> Result<(), Error> {
    let (store_name, key) = split_path(filename);
    let db = get_db().await?;

    let transaction = db
        .transaction(store_name)
        .with_mode(TransactionMode::Readwrite)
        .build()
        .map_err(|e| {
            log::error!("could not open write transaction: {}", e);
            Error::OpenFailed
        })?;

    let store = transaction.object_store(store_name).map_err(|e| {
        log::error!("could not open store '{}': {}", store_name, e);
        Error::OpenFailed
    })?;

    store.put(data).with_key(key).await.map_err(|e| {
        log::error!(
            "put failed for key '{}' in store '{}': {}",
            key,
            store_name,
            e
        );
        Error::OpenFailed
    })?;

    // commit() is best-effort: the transaction auto-commits when it goes out
    // of scope, and some IDB implementations raise InvalidStateError if you
    // call commit() after the request already settled.
    if let Err(e) = transaction.commit().await {
        log::warn!("commit returned an error (likely harmless): {}", e);
    }

    Ok(())
}

pub async fn write(filename: &str, data: String) {
    if let Err(e) = awrite(filename, data).await {
        log::error!("write('{}') failed: {}", filename, e);
    }
}

// ---------------------------------------------------------------------------
// Read
// ---------------------------------------------------------------------------

async fn aread(filename: &str) -> Result<String, Error> {
    let (store_name, key) = split_path(filename);
    let db = get_db().await?;

    let transaction = db
        .transaction(store_name)
        .with_mode(TransactionMode::Readonly)
        .build()
        .map_err(|e| {
            log::error!("could not open read transaction: {}", e);
            Error::OpenFailed
        })?;

    let store = transaction.object_store(store_name).map_err(|e| {
        log::error!("could not open store '{}': {}", store_name, e);
        Error::OpenFailed
    })?;

    match store.get(key).await.map_err(|e| {
        log::error!(
            "get failed for key '{}' in store '{}': {}",
            key,
            store_name,
            e
        );
        Error::OpenFailed
    })? {
        Some(value) => Ok(value),
        None => Err(Error::OpenFailed),
    }
}

pub async fn read(filename: &str) -> Result<String, Error> {
    aread(filename).await
}

// ---------------------------------------------------------------------------
// Cache hit check
// ---------------------------------------------------------------------------

async fn ahit_cache(filename: &str) -> bool {
    let (store_name, key) = split_path(filename);
    let db = match get_db().await {
        Ok(db) => db,
        Err(_) => return false,
    };

    let transaction = match db
        .transaction(store_name)
        .with_mode(TransactionMode::Readonly)
        .build()
    {
        Ok(t) => t,
        Err(e) => {
            log::error!("could not open transaction: {}", e);
            return false;
        }
    };

    let store = match transaction.object_store(store_name) {
        Ok(s) => s,
        Err(e) => {
            log::error!("could not open store '{}': {}", store_name, e);
            return false;
        }
    };

    store
        .get::<String, _, _>(key)
        .await
        .ok()
        .flatten()
        .is_some()
}

#[allow(dead_code)]
pub async fn hit_cache(filename: &str) -> bool {
    ahit_cache(filename).await
}
