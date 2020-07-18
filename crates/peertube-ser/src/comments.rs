use serde::Deserialize;

use super::common::Channel;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Comment {
    pub id: u64,
    pub url: String,
    pub text: String,
    pub threadId: i64,
    pub videoId: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deleted: Option<String>,
    pub isDeleted: bool,
    pub account: Channel,
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
    use serde_json::from_str;

    #[test]
    fn comments() {
        let test_data = r#"{"total":17,"data":[{"id":1,"url":"https://instance/videos/watch/UUID/comments/42","text":"Comment Text","threadId":1,"inReplyToCommentId":null,"videoId":5,"createdAt":"Some Date","updatedAt":"Some date","deletedAt":null,"isDeleted":false,"totalRepliesFromVideoAuthor":0,"totalReplies":0,"account":{"url":"https://SomeInstance","name":"Some name","host":"Some instance","avatar":null,"id":2,"hostRedundancyAllowed":false,"followingCount":0,"followersCount":0,"createdAt":"Some date","updatedAt":"Some date","displayName":"display","description":null}}]}"#;
        let _: Comments = from_str(test_data).unwrap();
    }
}
