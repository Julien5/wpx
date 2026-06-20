use crate::{cache, error::GenericResult, inputpoint::InputPoint, parameters::Parameters};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SmallDataset {
    parameters: Parameters,
    controls: Vec<InputPoint>,
}

impl SmallDataset {
    pub fn from_string(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    pub fn as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }
}

pub async fn write_userdata(
    parameters: &Parameters,
    controls: &Vec<InputPoint>,
) -> GenericResult<()> {
    let data = SmallDataset {
        parameters: parameters.clone(),
        controls: controls.clone(),
    };
    cache::write(
        &cache::Location::UserData,
        "parameters",
        data.as_string().unwrap(),
    )
    .await
}

pub async fn read_userdata() -> Option<SmallDataset> {
    match cache::read(&cache::Location::UserData, &"parameters-controls").await {
        Ok(bytes) => match SmallDataset::from_string(&bytes) {
            Ok(d) => Some(d),
            Err(e) => {
                log::error!("coud not read data {:?}", e);
                None
            }
        },
        Err(e) => {
            log::error!("coud not read data {:?}", e);
            None
        }
    }
}
