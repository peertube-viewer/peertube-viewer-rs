use serde::Deserialize;
use time::OffsetDateTime;

use super::common::{dates_deser, Channel, VideoState};

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct IdentifiedLabel {
    pub id: Option<u64>,
    pub label: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Language {
    pub id: Option<String>,
    pub label: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ScheduledUpdate {
    pub privacy: i64,
    pub updatedAt: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct UserHistory {
    pub currentTime: i64,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Video {
    pub uuid: String,
    #[serde(with = "dates_deser")]
    pub createdAt: OffsetDateTime,
    #[serde(with = "dates_deser")]
    pub publishedAt: OffsetDateTime,
    pub description: Option<String>,
    pub duration: u64,
    #[serde(default)]
    pub isLocal: bool,
    pub name: String,
    pub thumbnailPath: Option<String>,
    pub previewPath: Option<String>,
    pub embedPath: Option<String>,
    pub views: u64,
    pub likes: u64,
    pub dislikes: u64,
    pub nsfw: bool,

    #[serde(default)]
    pub isLive: bool,
    #[serde(default)]
    pub state: VideoState,
    pub account: Channel,
    pub channel: Channel,
    pub category: IdentifiedLabel,
    pub licence: IdentifiedLabel,
    pub language: Language,
    pub privacy: IdentifiedLabel,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Videos {
    pub total: usize,
    pub data: Vec<Video>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deser() {
        let test_data = r##"{"total":0,"data":[{"id":0,"uuid":"string","createdAt":"2018-11-15T17:58:28.154Z","publishedAt":"2018-11-15T17:58:28.154Z","updatedAt":"string","originallyPublishedAt":"2018-11-15T17:58:28.154Z","category":{"id":0,"label":"string"},"licence":{"id":0,"label":"string"},"language":{"id":"string","label":"string"},"privacy":{"id":1,"label":"string"},"description":"string","duration":0,"isLocal":true,"name":"string","thumbnailPath":"string","previewPath":"string","embedPath":"string","views":0,"likes":0,"dislikes":0,"nsfw":true,"waitTranscoding":true,"state":{"id":1,"label":"string"},"scheduledUpdate":{"privacy":1,"updateAt":"2020-04-22"},"blacklisted":true,"blacklistedReason":"string","account":{"id":0,"name":"string","displayName":"string","url":"string","host":"string","avatar":{"path":"string","createdAt":"2018-11-15T17:58:28.154Z","updatedAt":"string"}},"channel":{"id":0,"name":"string","displayName":"string","url":"string","host":"string","avatar":{"path":"string","createdAt":"2018-11-15T17:58:28.154Z","updatedAt":"string"}},"userHistory":{"currentTime":0}}]}"##;

        let _: Videos = serde_json::from_str(test_data).unwrap();
    }
}
