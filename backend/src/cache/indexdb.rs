use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;

/// Represents the distinct databases in your application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexdbLocation {
    UserData,
    OsmCache,
}

impl IndexdbLocation {
    pub fn from_location(b: &Location) -> Self {
        match b {
            Location::UserData => Self::UserData,
            Location::OsmCache => Self::OsmCache,
        }
    }
    fn version(&self) -> u8 {
        match self {
            Self::UserData => 1,
            Self::OsmCache => 1,
        }
    }
    fn name(&self) -> String {
        match self {
            Self::UserData => format!("{}", "UserData"),
            Self::OsmCache => format!("{}", "OsmCache"),
        }
    }

    /// Every database needs at least one object store to hold key-value data.
    /// We use a standard name across all databases for simplicity.
    fn store(&self) -> String {
        format!("{}", "store")
    }
}

use crate::cache::Location;

use thiserror::Error;
#[derive(Error, Debug, Clone)]
pub enum IndexdbError {
    #[error("OpenFailed")]
    OpenFailed,
    #[error("ReadFailed")]
    ReadFailed,
    #[error("WriteFailed")]
    WriteFailed,
}

async fn opendb(database: IndexdbLocation) -> Result<Database, IndexdbError> {
    match Database::open(database.name())
        .with_version(database.version())
        .with_on_upgrade_needed(move |event, db| {
            // Convert versions from floats to integers to allow using them in match expressions
            let old = event.old_version() as u64;
            let new = event.new_version().map(|v| v as u64);
            log::info!("old: {:?}, new: {:?}", old, new);
            /*
             * Migration logic come here, typically with match pattern:
             * Example:
             * match (old, new) {
             *   (0, Some(1)) => ..
             *   (prev, Some(2)) => ..
             * If the store name has changed, one could delete the old one:
             *   let _ = db.delete_object_store("store"); // old store name !
             * and create the new one:
             *   db.create_object_store("my_other_store").build()?;
             *   _ => {}
             * }
             */
            let store = database.store();
            // This logic runs whenever the requested version is higher than the existing version
            if !db.object_store_names().any(|n| n == store) {
                match db.create_object_store(&store).build() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("failed to create store: {} because {:?}", store, e);
                    }
                }
            }

            Ok(())
        })
        .await
    {
        Ok(db) => Ok(db),
        Err(e) => {
            log::info!("could not open db {}", e);
            Err(IndexdbError::OpenFailed)
        }
    }
}

async fn awrite(
    database: &IndexdbLocation,
    filename: &str,
    data: String,
) -> Result<(), IndexdbError> {
    log::trace!("db - write {}: {}", database.name(), filename);
    let db = match opendb(database.clone()).await {
        Ok(db) => db,
        Err(e) => {
            log::error!("could not open db: {}", e);
            return Err(e);
        }
    };
    // Populate some data
    let transaction = db
        .transaction(database.store())
        .with_mode(TransactionMode::Readwrite)
        .build();

    match transaction {
        Ok(t) => {
            let store = t.object_store(&database.store()).unwrap();
            match store.put(data).with_key(filename).await {
                Ok(s) => {
                    log::info!("write: {}", s);
                }
                Err(e) => {
                    log::info!("could not put data because {}", e);
                    return Err(IndexdbError::WriteFailed);
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
                    log::info!("could not commit because {}", e);
                }
            }
        }
        Err(e) => {
            log::info!("could not open transaction because {}", e);
            return Err(IndexdbError::WriteFailed);
        }
    }
    Ok(())
}

pub async fn write(
    database: &IndexdbLocation,
    filename: &str,
    data: String,
) -> Result<(), IndexdbError> {
    awrite(database, filename, data).await
}

async fn aread(database: &IndexdbLocation, filename: &str) -> Result<String, IndexdbError> {
    let db = match opendb(database.clone()).await?;
	let transaction = db
        .transaction(database.store())
        .with_mode(TransactionMode::Readonly)
        .build()
        .map_err(|_| IndexdbError::ReadFailed)?;
    let store = transaction.object_store(&database.store()).unwrap();
    let data = store.get(&filename).await.unwrap();
    match data {
        Some(bytes) => Ok(bytes),
        None => Err(IndexdbError::ReadFailed),
    }
}

pub async fn read(database: &IndexdbLocation, filename: &str) -> Result<String, IndexdbError> {
    aread(database, filename).await
}

pub async fn allfiles(database: &IndexdbLocation) -> Result<Vec<String>, IndexdbError> {
    let db = opendb(database.clone()).await?;
    let transaction = db
        .transaction(database.store())
        .with_mode(TransactionMode::Readonly)
        .build()
        .map_err(|_| IndexdbError::ReadFailed)?;
    let store = transaction.object_store(&database.store()).unwrap();
    let iter = store
        .get_all_keys::<String>()
        .await
        .map_err(|_| IndexdbError::ReadFailed)?;
    iter.collect::<Result<Vec<_>, _>>()
        .map_err(|_| IndexdbError::ReadFailed)
}
