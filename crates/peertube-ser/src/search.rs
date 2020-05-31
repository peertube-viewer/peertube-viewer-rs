use serde::{Deserialize, Serialize};

use super::common::Avatar;

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub displayName: String,
    pub url: String,
    pub host: String,
    pub Avatar: Option<Avatar>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct IdentifiedLabel {
    pub id: Option<i64>,
    pub label: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Language {
    pub id: Option<String>,
    pub label: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ScheduledUpdate {
    pub privacy: Option<i64>,
    pub updatedAt: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct UserHistory {
    pub currentTime: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Video {
    pub uuid: Option<String>,
    pub createdAt: Option<String>,
    pub publishedAt: Option<String>,
    pub updatedAt: Option<String>,
    pub originallyPublishedAt: Option<String>,
    pub description: Option<String>,
    pub duration: Option<i64>,
    pub isLocal: Option<bool>,
    pub name: Option<String>,
    pub thumbnailPath: Option<String>,
    pub previewPath: Option<String>,
    pub embedPath: Option<String>,
    pub views: Option<i64>,
    pub likes: Option<i64>,
    pub dislikes: Option<i64>,
    pub nsfw: Option<bool>,
    pub waitTranscoding: Option<bool>,
    pub account: Channel,
    pub channel: Channel,
    pub category: IdentifiedLabel,
    pub licence: IdentifiedLabel,
    pub language: Language,
    pub privacy: IdentifiedLabel,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Videos {
    pub total: Option<i64>,
    pub data: Vec<Video>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn deser() {
        let test_data = r##"{"total":0,"data":[{"id":0,"uuid":"string","createdAt":"string","publishedAt":"string","updatedAt":"string","originallyPublishedAt":"string","category":{"id":0,"label":"string"},"licence":{"id":0,"label":"string"},"language":{"id":"string","label":"string"},"privacy":{"id":1,"label":"string"},"description":"string","duration":0,"isLocal":true,"name":"string","thumbnailPath":"string","previewPath":"string","embedPath":"string","views":0,"likes":0,"dislikes":0,"nsfw":true,"waitTranscoding":true,"state":{"id":1,"label":"string"},"scheduledUpdate":{"privacy":1,"updateAt":"2020-04-22"},"blacklisted":true,"blacklistedReason":"string","account":{"id":0,"name":"string","displayName":"string","url":"string","host":"string","avatar":{"path":"string","createdAt":"string","updatedAt":"string"}},"channel":{"id":0,"name":"string","displayName":"string","url":"string","host":"string","avatar":{"path":"string","createdAt":"string","updatedAt":"string"}},"userHistory":{"currentTime":0}}]}"##;

        let _: Videos = from_str(test_data).unwrap();
    }
}
