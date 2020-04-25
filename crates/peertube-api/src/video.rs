use chrono::{DateTime, FixedOffset};
use derive_getters::Getters;
use tokio::sync::Mutex;

use std::error;

#[cfg(not(feature = "send"))]
use std::rc::Rc as FeaturedRc;
#[cfg(feature = "send")]
use std::sync::Arc as FeaturedRc;

use crate::instance::Instance;
use peertube_ser::{search, video};

#[derive(Clone, Debug, Getters)]
struct File {
    magnetUri: String,
    resoltion_id: i64,
    resolution: String,
    size: u64,
    torrentUrl: String,
    torrentDownloadUrl: String,
    fileUrl: String,
    fileDownloadUrl: String,
}

#[derive(Clone, Debug, Getters)]
pub struct Resolution {
    id: i64,
    label: String,
    size: u64,
}

impl Resolution {
    fn from_file(f: &File) -> Resolution {
        Resolution {
            id: f.resoltion_id,
            label: f.resolution.clone(),
            size: f.size,
        }
    }
}

impl From<video::File> for File {
    fn from(v: video::File) -> File {
        File {
            magnetUri: v.magnetUri,
            resoltion_id: v.resolution.id,
            resolution: v.resolution.label,
            size: if v.size > 0 { v.size as u64 } else { 0 },
            torrentUrl: v.torrentUrl,
            torrentDownloadUrl: v.torrentDownloadUrl,
            fileUrl: v.fileUrl,
            fileDownloadUrl: v.fileDownloadUrl,
        }
    }
}

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
    files: Mutex<Option<Vec<File>>>,
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
                files: Mutex::new(None),
                channel: v.channel,
                account: v.account,
            })
        } else {
            None
        }
    }

    pub fn watch_url(&self) -> String {
        let mut video_url = self.instance.host().to_string();
        video_url.push_str("/videos/watch/");
        video_url.push_str(&self.uuid);
        video_url
    }
    pub async fn description(&self) -> Result<Option<String>, Box<dyn error::Error>> {
        let mut guard = self.description.lock().await;
        if guard.is_none() {
            *guard = Some(self.instance.video_description(&self.uuid).await?);
        }
        Ok(guard.as_ref().unwrap().clone())
    }
    pub async fn resolutions(&self) -> Result<Vec<Resolution>, Box<dyn error::Error>> {
        let mut guard = self.files.lock().await;
        if guard.is_none() {
            *guard = Some(
                self.instance
                    .video_complete(&self.uuid)
                    .await?
                    .drain(..)
                    .map(|v| v.into())
                    .collect(),
            );
        }

        let resolutions = guard
            .as_ref()
            .unwrap()
            .iter()
            .map(|file| Resolution::from_file(file))
            .collect();

        //TODO Find a way to remove the unnecessary clone
        Ok(resolutions)
    }

    pub fn channel_display(&self) -> &str {
        &self.channel.displayName
    }

    pub fn account_display(&self) -> &str {
        &self.account.displayName
    }

    pub async fn resolution_url(&self, mut id: usize) -> Result<String, Box<dyn error::Error>> {
        let mut guard = self.files.lock().await;
        if guard.is_none() {
            *guard = Some(
                self.instance
                    .video_complete(&self.uuid)
                    .await?
                    .drain(..)
                    .map(|v| v.into())
                    .collect(),
            );
        }

        Ok(guard.as_ref().unwrap()[id].fileDownloadUrl.clone())
    }

    pub fn host(&self) -> &str {
        &self.account.host
    }
}

fn floor_default(i: Option<i64>) -> u64 {
    i.map(|count| if count < 0 { 0 } else { count as u64 })
        .unwrap_or_default()
}
