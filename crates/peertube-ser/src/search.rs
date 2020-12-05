#![allow(clippy::redundant_pattern_matching)] // Suppress warnings caused by nanoserde

use nanoserde::DeJson;

use super::common::Channel;

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct IdentifiedLabel {
    pub id: Option<u64>,
    pub label: String,
}

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Language {
    pub id: Option<String>,
    pub label: String,
}

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct ScheduledUpdate {
    pub privacy: i64,
    pub updatedAt: String,
}

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct UserHistory {
    pub currentTime: i64,
}

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Video {
    pub uuid: String,
    pub createdAt: String,
    pub publishedAt: String,
    pub updatedAt: String,
    pub description: Option<String>,
    pub duration: u64,
    pub isLocal: bool,
    pub name: String,
    pub thumbnailPath: Option<String>,
    pub previewPath: Option<String>,
    pub embedPath: Option<String>,
    pub views: u64,
    pub likes: u64,
    pub dislikes: u64,
    pub nsfw: bool,
    pub account: Channel,
    pub channel: Channel,
    pub category: IdentifiedLabel,
    pub licence: IdentifiedLabel,
    pub language: Language,
    pub privacy: IdentifiedLabel,
}

#[derive(DeJson, Debug)]
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
        let test_data = r##"{"total":0,"data":[{"id":0,"uuid":"string","createdAt":"string","publishedAt":"string","updatedAt":"string","originallyPublishedAt":"string","category":{"id":0,"label":"string"},"licence":{"id":0,"label":"string"},"language":{"id":"string","label":"string"},"privacy":{"id":1,"label":"string"},"description":"string","duration":0,"isLocal":true,"name":"string","thumbnailPath":"string","previewPath":"string","embedPath":"string","views":0,"likes":0,"dislikes":0,"nsfw":true,"waitTranscoding":true,"state":{"id":1,"label":"string"},"scheduledUpdate":{"privacy":1,"updateAt":"2020-04-22"},"blacklisted":true,"blacklistedReason":"string","account":{"id":0,"name":"string","displayName":"string","url":"string","host":"string","avatar":{"path":"string","createdAt":"string","updatedAt":"string"}},"channel":{"id":0,"name":"string","displayName":"string","url":"string","host":"string","avatar":{"path":"string","createdAt":"string","updatedAt":"string"}},"userHistory":{"currentTime":0}}]}"##;

        let _: Videos = DeJson::deserialize_json(test_data).unwrap();
    }
}
