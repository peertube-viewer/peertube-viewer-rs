use serde::{Deserialize, Serialize};

use super::common::Avatar;

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Account {
    pub url: String,
    pub name: String,
    pub host: String,
    pub avatar: Option<Avatar>,
    pub id: i64,
    pub followingCount: i64,
    pub followersCount: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub displayName: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Comment {
    pub id: i64,
    pub url: String,
    pub text: Option<String>,
    pub threadId: i64,
    pub inReplyToCommentId: Option<String>,
    pub videoId: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deleted: Option<String>,
    pub isDeleted: bool,
    pub account: Option<Account>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Comments {
    pub total: Option<i64>,
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
