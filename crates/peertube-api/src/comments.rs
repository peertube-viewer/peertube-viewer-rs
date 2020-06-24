use crate::common::Channel;
use crate::Instance;
use std::convert::TryFrom;
use std::rc::Rc;

pub struct Comment {
    content: String,
    url: String,
    author: Option<Channel>,
}

impl TryFrom<peertube_ser::comments::Comment> for Comment {
    type Error = ();
    fn try_from(comment: peertube_ser::comments::Comment) -> Result<Self, ()> {
        Ok(Comment {
            content: if let Some(t) = comment.text {
                t
            } else {
                return Err(());
            },
            url: comment.url,
            author: comment.account.map(|c| c.into()),
        })
    }
}
