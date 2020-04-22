use chrono::{DateTime, FixedOffset};
use tokio::sync::Mutex;

use std::error;
#[cfg(not(feature = "send"))]
use std::rc::Rc as FeaturedRc;
#[cfg(feature = "send")]
use std::sync::Arc as FeaturedRc;

use crate::instance::Instance;

pub struct Video {
    instance: FeaturedRc<Instance>,
    name: String,
    uuid: String,
    duration: u64,
    published: Option<DateTime<FixedOffset>>,
    short_desc: Option<String>,
    description: Mutex<Option<Option<String>>>,
}

impl Video {
    pub fn new(
        instance: FeaturedRc<Instance>,
        name: String,
        uuid: String,
        duration: u64,
        published: String,
        short_desc: Option<String>,
    ) -> Video {
        Video {
            instance,
            name,
            uuid,
            duration,
            published: DateTime::parse_from_rfc3339(&published).ok(),
            short_desc,
            description: Mutex::new(None),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn uuid(&self) -> &String {
        &self.uuid
    }

    pub fn short_description(&self) -> &Option<String> {
        &self.short_desc
    }

    pub fn duration(&self) -> u64 {
        self.duration
    }

    pub fn published(&self) -> &Option<DateTime<FixedOffset>> {
        &self.published
    }

    pub async fn description(&self) -> Result<Option<String>, Box<dyn error::Error>> {
        let mut guard = self.description.lock().await;
        if guard.is_none() {
            *guard = Some(self.instance.video_description(&self.uuid).await?);
        }
        Ok(guard.as_ref().unwrap().clone())
    }
}
