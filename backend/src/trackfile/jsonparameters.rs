use crate::{
    cache,
    error::GenericResult,
    parameters::Parameters,
    trackfile::{v0, TrackFile},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JsonParameters {
    #[serde(
        deserialize_with = "deserialize_versioned_parameters",
        serialize_with = "serialize_versioned_parameters"
    )]
    pub parameters: Parameters,
}

#[derive(Deserialize)]
#[serde(tag = "version")]
enum VersionedParameters {
    #[serde(rename = "0")]
    V0(v0::parameters::Parameters),
    #[serde(rename = "1")]
    V1(Parameters),
}

fn serialize_versioned_parameters<S>(
    parameters: &Parameters,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Defined locally inside the function so it disappears outside of it
    #[derive(Serialize)]
    struct Wrapper<'a> {
        version: &'static str,
        #[serde(flatten)]
        parameters: &'a Parameters,
    }

    Wrapper {
        version: "1",
        parameters,
    }
    .serialize(serializer)
}

fn deserialize_versioned_parameters<'de, D>(deserializer: D) -> Result<Parameters, D::Error>
where
    D: Deserializer<'de>,
{
    match VersionedParameters::deserialize(deserializer)? {
        VersionedParameters::V0(p) => Ok(p.into()),
        VersionedParameters::V1(p) => Ok(p),
    }
}

static JSONPARAMETERS_FILENAME: &'static str = "parameters.json";
impl JsonParameters {
    pub fn from_string(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    fn as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    pub async fn write(&self, trackfile: &TrackFile) -> GenericResult<()> {
        cache::write(
            &cache::Location::UserData,
            &super::basename(trackfile.number, JSONPARAMETERS_FILENAME),
            self.as_string().unwrap(),
        )
        .await
    }

    pub async fn read(trackfile: &TrackFile) -> GenericResult<Self> {
        let bytes = cache::read(
            &cache::Location::UserData,
            &super::basename(trackfile.number, JSONPARAMETERS_FILENAME),
        )
        .await?;
        let data = JsonParameters::from_string(&bytes)?;
        Ok(data)
    }
}
