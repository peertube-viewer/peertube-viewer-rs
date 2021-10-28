use crate::common::Channel;
use std::convert::TryFrom;
use time::OffsetDateTime;

pub struct Comment {
    content: String,
    url: String,
    created_at: OffsetDateTime,
    author: Channel,
}

impl TryFrom<peertube_ser::comments::Comment> for Comment {
    type Error = ();
    fn try_from(comment: peertube_ser::comments::Comment) -> Result<Self, ()> {
        match (comment.isDeleted, comment.url, comment.account) {
            (false, Some(url), Some(account)) => Ok(Comment {
                content: comment.text,
                url,
                created_at: comment.createdAt,
                author: account.into(),
            }),
            _ => Err(()),
        }
    }
}

impl Comment {
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn author_display_name(&self) -> &str {
        &self.author.display_name
    }

    pub fn author_host(&self) -> &str {
        &self.author.host
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}
