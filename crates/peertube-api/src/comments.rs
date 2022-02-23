// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

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
