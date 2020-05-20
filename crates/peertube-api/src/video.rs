use chrono::{DateTime, FixedOffset};
use tokio::sync::Mutex;

#[cfg(not(feature = "send"))]
use std::rc::Rc as FeaturedRc;
#[cfg(feature = "send")]
use std::sync::Arc as FeaturedRc;

use crate::error;
use crate::instance::Instance;
use peertube_ser::{search, video};

#[derive(Clone, Debug)]
struct File {
    magnet_uri: String,
    resoltion_id: i64,
    resolution: String,
    size: u64,
    torrent_url: String,
    torrent_download_url: String,
    webseed_url: String,
    download_url: String,
}

#[allow(unused)]
impl File {
    fn magnet_uri(&self) -> &str {
        &self.magnet_uri
    }
    fn resoltion_id(&self) -> &i64 {
        &self.resoltion_id
    }
    fn resolution(&self) -> &str {
        &self.resolution
    }
    fn size(&self) -> &u64 {
        &self.size
    }
    fn torrent_url(&self) -> &str {
        &self.torrent_url
    }
    fn torrent_download_url(&self) -> &str {
        &self.torrent_download_url
    }
    fn webseed_url(&self) -> &str {
        &self.webseed_url
    }
    fn download_url(&self) -> &str {
        &self.download_url
    }
}

#[derive(Clone, Debug)]
struct Channel {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub url: String,
    pub host: String,
}

#[allow(unused)]
impl Channel {
    fn id(&self) -> &i64 {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn display_name(&self) -> &str {
        &self.display_name
    }
    fn url(&self) -> &str {
        &self.url
    }
    fn host(&self) -> &str {
        &self.host
    }
}

#[derive(Clone, Debug)]
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

#[allow(unused)]
impl Resolution {
    pub fn id(&self) -> &i64 {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn size(&self) -> &u64 {
        &self.size
    }
}

impl From<video::File> for File {
    fn from(v: video::File) -> File {
        File {
            magnet_uri: v.magnetUri,
            resoltion_id: v.resolution.id,
            resolution: v.resolution.label,
            size: if v.size > 0 { v.size as u64 } else { 0 },
            torrent_url: v.torrentUrl,
            torrent_download_url: v.torrentDownloadUrl,
            webseed_url: v.fileUrl,
            download_url: v.fileDownloadUrl,
        }
    }
}

impl From<search::Channel> for Channel {
    fn from(c: search::Channel) -> Channel {
        Channel {
            id: c.id,
            name: c.name,
            display_name: c.displayName,
            url: c.url,
            host: c.host,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Description {
    None,
    FetchedNone,
    Fetched(String),
}

impl Description {
    pub fn is_none(&self) -> bool {
        Description::None == *self
    }

    pub fn to_option(&self) -> Option<String> {
        if let Description::Fetched(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }
}

/// Handle to a video
pub struct Video {
    instance: FeaturedRc<Instance>,
    name: String,
    uuid: String,
    duration: u64,
    views: u64,
    likes: u64,
    nsfw: bool,
    dislikes: u64,
    published: Option<DateTime<FixedOffset>>,
    short_desc: Option<String>,
    description: Mutex<Description>,
    files: Mutex<Option<Vec<File>>>,
    channel: Channel,
    account: Channel,
}

#[allow(unused)]
impl Video {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn uuid(&self) -> &str {
        &self.uuid
    }
    pub fn short_desc(&self) -> Option<&str> {
        self.short_desc.as_ref().map(|s| s.as_str())
    }
    pub fn published(&self) -> Option<&DateTime<FixedOffset>> {
        self.published.as_ref()
    }
    pub fn duration(&self) -> u64 {
        self.duration
    }
    pub fn views(&self) -> u64 {
        self.views
    }
    pub fn likes(&self) -> u64 {
        self.likes
    }
    pub fn nsfw(&self) -> bool {
        self.nsfw
    }
    pub fn dislikes(&self) -> u64 {
        self.dislikes
    }

    pub fn host(&self) -> &str {
        &self.account.host
    }

    pub fn channel_display_name(&self) -> &str {
        &self.channel.display_name
    }

    pub fn channel_display(&self) -> &str {
        &self.channel.display_name
    }

    pub fn account_display(&self) -> &str {
        &self.account.display_name
    }
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
                nsfw: v.nsfw.unwrap_or(false),
                published: v
                    .publishedAt
                    .map(|d| DateTime::parse_from_rfc3339(&d).ok())
                    .flatten(),
                short_desc: v.description,
                description: Mutex::new(Description::None),
                files: Mutex::new(None),
                channel: v.channel.into(),
                account: v.account.into(),
            })
        } else {
            None
        }
    }
    pub fn from(i: &FeaturedRc<Instance>, mut v: video::Video) -> Video {
        Video {
            instance: i.clone(),
            name: v.name,
            uuid: v.uuid,
            duration: floor_default(v.duration),
            likes: floor_default(v.likes),
            dislikes: floor_default(v.dislikes),
            views: floor_default(v.views),
            nsfw: v.nsfw.unwrap_or(false),
            published: v
                .publishedAt
                .map(|d| DateTime::parse_from_rfc3339(&d).ok())
                .flatten(),
            short_desc: v.description,
            description: Mutex::new(Description::None),
            files: Mutex::new(Some(v.files.drain(..).map(|v| v.into()).collect())),
            channel: v.channel.into(),
            account: v.account.into(),
        }
    }

    /// Get the url to watch the video from a browser
    pub fn watch_url(&self) -> String {
        let mut video_url = "https://".to_string();
        video_url.push_str(&self.account.host);
        video_url.push_str("/videos/watch/");
        video_url.push_str(&self.uuid);
        video_url
    }

    /// Get the full description
    /// During the lifetime of the struct, the description will be fetched only once and the result
    /// is stored and re-used
    pub async fn description(&self) -> error::Result<Option<String>> {
        let mut guard = self.description.lock().await;
        if guard.is_none() {
            *guard = match self.fetch_description().await? {
                Some(s) => Description::Fetched(s),
                None => Description::FetchedNone,
            };
        }
        Ok(guard.to_option())
    }

    async fn fetch_description(&self) -> error::Result<Option<String>> {
        self.instance.video_description(&self.uuid).await
    }

    /// Fetch the description but don't return it to avoid an unnecessary copy
    /// The result is store withing the struct
    ///
    /// Used to asynchronously load the description for later use
    pub async fn load_description(&self) -> error::Result<()> {
        let mut guard = self.description.lock().await;
        if guard.is_none() {
            *guard = match self.fetch_description().await? {
                Some(s) => Description::Fetched(s),
                None => Description::FetchedNone,
            };
        }
        Ok(())
    }

    /// Fetch the available resolutions but don't return it to avoid an unnecessary copy
    /// The result is store withing the struct
    ///
    /// Used to asynchronously load the resolutions for later use
    pub async fn load_resolutions(&self) -> error::Result<()> {
        let mut guard = self.files.lock().await;
        if guard.is_none() {
            *guard = Some(self.fetch_files().await?);
        }
        Ok(())
    }

    async fn fetch_files(&self) -> error::Result<Vec<File>> {
        Ok(self
            .instance
            .video_complete(&self.uuid)
            .await?
            .drain(..)
            .map(|v| v.into())
            .collect())
    }

    /// Get the available resolutions
    /// During the lifetime of the struct, the resolutions will be fetched only once and the result
    /// is stored and re-used
    pub async fn resolutions(&self) -> error::Result<Vec<Resolution>> {
        let mut guard = self.files.lock().await;
        if guard.is_none() {
            *guard = Some(self.fetch_files().await?);
        }

        let resolutions = guard
            .as_ref()
            .unwrap()
            .iter()
            .map(|file| Resolution::from_file(file))
            .collect();

        Ok(resolutions)
    }

    /// Get a url for a given resolution
    pub async fn resolution_url(&self, id: usize) -> String {
        let guard = self.files.lock().await;
        if let Some(res) = guard.as_ref() {
            res[id].download_url.clone()
        } else {
            panic!("Resolution hasn't been fetched");
        }
    }

    /// Get a torrent url for a given resolution
    pub async fn torrent_url(&self, id: usize) -> String {
        let guard = self.files.lock().await;
        if let Some(res) = guard.as_ref() {
            res[id].torrent_download_url.clone()
        } else {
            panic!("Resolution hasn't been fetched");
        }
    }
}

fn floor_default(i: Option<i64>) -> u64 {
    i.map(|count| if count < 0 { 0 } else { count as u64 })
        .unwrap_or_default()
}
