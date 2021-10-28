use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Avatar {
    pub path: String,
    pub createdAt: Option<String>,
    pub updatedAt: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub displayName: String,

    #[serde(default)]
    pub url: Option<String>,
    pub host: String,
    pub Avatar: Option<Avatar>,
}

/// Structure used to deserialize the state of a video
#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct VideoState {
    pub id: u16,
    pub label: String,
}

pub mod dates_deser {
    use serde::{de::Error, Deserialize, Deserializer};
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<OffsetDateTime, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        OffsetDateTime::parse(&time, &Rfc3339).map_err(D::Error::custom)
    }
}
