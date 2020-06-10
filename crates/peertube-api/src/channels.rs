use chrono::{DateTime, FixedOffset};

use peertube_ser::channels;

pub struct Channel {
    id: (String, u64), // Ids are not valable accross instances
    //so we need to keep the instance where this ID is valable
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

    pub fn handle(&self) -> String {
        format!("{}@{}", self.name, self.host)
    }

    pub fn rss(&self) -> String {
        format!(
            "{}/feeds/videos.xml?videoChannelId={}",
            self.id.0, self.id.1
        )
    }

    pub fn atom(&self) -> String {
        format!(
            "{}/feeds/videos.atom?videoChannelId={}",
            self.id.0, self.id.1
        )
    }
}

impl Channel {
    pub fn maybe_from(c: channels::Channel, source_instance: String) -> Option<Channel> {
        Some(Channel {
            id: if c.id > 0 {
                (source_instance, c.id as u64)
            } else {
                (source_instance, 0)
            },
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
