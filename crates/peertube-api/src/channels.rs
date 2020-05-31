use chrono::{DateTime, FixedOffset};

use peertube_ser::channels;

pub struct Channel {
    id: u64,
    name: String,
    display_name: String,
    description: Option<String>,
    host: String,
    followers: u64,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

#[allow(unused)]
impl Channel {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    pub fn followers(&self) -> u64 {
        self.followers
    }
    pub fn host(&self) -> &str {
        &self.host
    }
    pub fn description(&self) -> &Option<String> {
        &self.description
    }
    pub fn created_at(&self) -> &DateTime<FixedOffset> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<FixedOffset> {
        &self.updated_at
    }
}

impl Channel {
    pub fn maybe_from(c: channels::Channel) -> Option<Channel> {
        Some(Channel {
            id: if c.id > 0 { c.id as u64 } else { 0 },
            followers: if c.followersCount > 0 {
                c.followersCount as u64
            } else {
                0
            },
            name: c.name,
            display_name: c.displayName,
            description: c.description,
            created_at: DateTime::parse_from_rfc3339(&c.createdAt).ok()?,
            updated_at: DateTime::parse_from_rfc3339(&c.createdAt).ok()?,
            host: c.host,
        })
    }
}
