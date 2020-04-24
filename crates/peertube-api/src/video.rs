use chrono::{DateTime, FixedOffset};
use derive_getters::Getters;
use tokio::sync::Mutex;

use std::error;

#[cfg(not(feature = "send"))]
use std::rc::Rc as FeaturedRc;
#[cfg(feature = "send")]
use std::sync::Arc as FeaturedRc;

use crate::instance::Instance;
use peertube_ser::search;

#[derive(Getters)]
pub struct Video {
    #[getter(skip)]
    instance: FeaturedRc<Instance>,
    name: String,
    uuid: String,
    duration: u64,
    views: u64,
    likes: u64,
    dislikes: u64,
    published: Option<DateTime<FixedOffset>>,
    short_desc: Option<String>,
    #[getter(skip)]
    description: Mutex<Option<Option<String>>>,
    #[getter(skip)]
    channel: search::Channel,
    #[getter(skip)]
    account: search::Channel,
}

impl Video {
    pub fn maybe_from(i: &FeaturedRc<Instance>, v: search::Video) -> Option<Video> {
        if let (Some(name), Some(uuid)) = (v.name, v.uuid) {
            Some(Video {
                instance: i.clone(),
                name,
                uuid,
                duration: floor_default(v.duration),
                likes: floor_default(v.likes),
                dislikes: floor_default(v.dislikes),
                views: floor_default(v.views),
                published: v
                    .publishedAt
                    .map(|d| DateTime::parse_from_rfc3339(&d).ok())
                    .flatten(),
                short_desc: v.description,
                description: Mutex::new(None),
                channel: v.channel,
                account: v.account,
            })
        } else {
            None
        }
    }

    pub async fn description(&self) -> Result<Option<String>, Box<dyn error::Error>> {
        let mut guard = self.description.lock().await;
        if guard.is_none() {
            *guard = Some(self.instance.video_description(&self.uuid).await?);
        }
        Ok(guard.as_ref().unwrap().clone())
    }

    pub fn channel_display_name(&self) -> &str {
        &self.channel.displayName
    }

    pub fn account_display_name(&self) -> &str {
        &self.account.displayName
    }

    pub fn host(&self) -> &str {
        &self.account.host
    }
}

fn floor_default(i: Option<i64>) -> u64 {
    i.map(|count| if count < 0 { 0 } else { count as u64 })
        .unwrap_or_default()
}
