use crate::{cache, error::GenericResult};

pub async fn write_parameters() -> GenericResult<()> {
    cache::write(&cache::Location::UserData, "foo", "bar".to_string()).await
}
