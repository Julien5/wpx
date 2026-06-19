use crate::{cache, error::GenericResult, parameters::Parameters};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct UserData {
    parameters: Parameters,
}

impl UserData {
    pub fn new() -> Self {
        Self {
            parameters: Parameters::default(),
        }
    }
    pub fn from_string(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    pub fn as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }
}

pub async fn write_parameters(parameters: &Parameters) -> GenericResult<()> {
    let data = UserData {
        parameters: parameters.clone(),
    };
    cache::write(
        &cache::Location::UserData,
        "parameters",
        data.as_string().unwrap(),
    )
    .await
}
