use serde::Deserialize;
use time::OffsetDateTime;

use super::common::{dates_deser, Channel};

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Comment {
    pub id: u64,

    pub url: Option<String>,
    pub text: String,
    pub threadId: i64,
    pub videoId: i64,
    #[serde(with = "dates_deser")]
    pub createdAt: OffsetDateTime,
    pub deleted: Option<String>,
    pub isDeleted: bool,
    pub account: Option<Channel>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Comments {
    pub total: usize,
    pub data: Vec<Comment>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments() {
        let test_data = r#"{"total":17,"data":[{"id":1,"url":"https://instance/videos/watch/UUID/comments/42","text":"Comment Text","threadId":1,"inReplyToCommentId":null,"videoId":5,"createdAt":"2018-11-15T17:58:28.154Z","updatedAt":"Some date","deletedAt":null,"isDeleted":false,"totalRepliesFromVideoAuthor":0,"totalReplies":0,"account":{"url":"https://SomeInstance","name":"Some name","host":"Some instance","avatar":null,"id":2,"hostRedundancyAllowed":false,"followingCount":0,"followersCount":0,"createdAt":"2018-11-15T17:58:28.154Z","updatedAt":"Some date","displayName":"display","description":null}}]}"#;
        let _: Comments = serde_json::from_str(test_data).unwrap();
    }
}
