use chrono::{DateTime, FixedOffset};
use std::sync::Arc;
use std::sync::Mutex;

use crate::common::Channel;
use crate::error::{self, Error};
use crate::instance::Instance;
use peertube_ser::{common::VideoState, search, video};

#[derive(Clone, Debug)]
struct File {
    magnet_uri: String,
    resoltion_id: u64,
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
    fn resoltion_id(&self) -> &u64 {
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

#[derive(Clone, Debug)]
pub struct StreamingPlaylist {
    id: u64,
    playlist_url: String,
}

#[allow(unused)]
impl StreamingPlaylist {
    pub fn playlist_url(&self) -> &str {
        &self.playlist_url
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl From<video::StreamingPlaylist> for StreamingPlaylist {
    fn from(v: video::StreamingPlaylist) -> StreamingPlaylist {
        StreamingPlaylist {
            id: v.id,
            playlist_url: v.playlistUrl,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Resolution {
    id: u64,
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
    pub fn id(&self) -> &u64 {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn size(&self) -> &u64 {
        &self.size
    }
}

#[derive(Debug, Clone)]
enum Description {
    None,
    FetchedNone,
    FetchedError(Error),
    Fetched(String),
}

#[derive(Debug, Clone)]
pub enum State {
    Published,
    ToTranscode,
    ToImport,
    WaitingForLive,
    LiveEnded,
    Unknown(u16, String),
}

impl From<VideoState> for State {
    fn from(i: VideoState) -> State {
        match i.id {
            1 => State::Published,
            2 => State::ToTranscode,
            3 => State::ToImport,
            4 => State::WaitingForLive,
            5 => State::LiveEnded,
            _ => State::Unknown(i.id, i.label),
        }
    }
}

impl Description {
    pub fn is_none(&self) -> bool {
        matches!(*self, Description::None)
    }

    pub fn to_option(&self) -> Option<String> {
        if let Description::Fetched(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
enum Files {
    None,
    FetchedError(Error),
    Fetched(Vec<File>, Vec<StreamingPlaylist>),
}

/// Handle to a video
pub struct Video {
    instance: Arc<Instance>,
    name: String,
    uuid: String,
    duration: u64,
    views: u64,
    likes: u64,
    nsfw: bool,
    is_live: bool,
    dislikes: u64,
    published: Option<DateTime<FixedOffset>>,
    short_desc: Option<String>,
    description: Mutex<Description>,
    files: Mutex<Files>,
    channel: Channel,
    account: Channel,
    state: State,
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
        self.short_desc.as_deref()
    }
    pub fn published(&self) -> Option<&DateTime<FixedOffset>> {
        self.published.as_ref()
    }
    pub fn duration(&self) -> u64 {
        self.duration
    }
    pub fn is_live(&self) -> bool {
        self.is_live
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

    pub fn state(&self) -> &State {
        &self.state
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
    pub fn from_search(i: &Arc<Instance>, v: search::Video) -> Video {
        Video {
            instance: i.clone(),
            name: v.name,
            uuid: v.uuid,
            duration: v.duration,
            likes: v.likes,
            dislikes: v.dislikes,
            views: v.views,
            nsfw: v.nsfw,
            is_live: v.isLive,
            published: DateTime::parse_from_rfc3339(&v.publishedAt).ok(),
            short_desc: v.description,
            description: Mutex::new(Description::None),
            files: Mutex::new(Files::None),
            channel: v.channel.into(),
            account: v.account.into(),
            state: v.state.into(),
        }
    }
    pub fn from_full(i: &Arc<Instance>, v: video::Video) -> Video {
        Video {
            instance: i.clone(),
            name: v.name,
            uuid: v.uuid,
            duration: v.duration,
            likes: v.likes,
            dislikes: v.dislikes,
            views: v.views,
            nsfw: v.nsfw,
            is_live: v.isLive,
            published: DateTime::parse_from_rfc3339(&v.publishedAt).ok(),
            short_desc: v.description,
            description: Mutex::new(Description::None),
            files: Mutex::new(Files::Fetched(
                v.files.into_iter().map(|v| v.into()).collect(),
                v.streamingPlaylists.into_iter().map(|v| v.into()).collect(),
            )),
            channel: v.channel.into(),
            account: v.account.into(),
            state: v.state.into(),
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
    pub fn description(&self) -> error::Result<Option<String>> {
        let mut guard = self.description.lock().unwrap();
        if guard.is_none() {
            *guard = match self.fetch_description() {
                Ok(Some(s)) => Description::Fetched(s),
                Ok(None) => Description::FetchedNone,
                Err(err) => Description::FetchedError(err),
            };
        }
        Ok(guard.to_option())
    }

    fn fetch_description(&self) -> error::Result<Option<String>> {
        self.instance.video_description(self.host(), &self.uuid)
    }

    /// Fetch the description but don't return it to avoid an unnecessary copy
    /// The result is store within the struct
    ///
    /// Used to asynchronously load the description for later use
    pub fn load_description(&self) -> error::Result<()> {
        let mut guard = self.description.lock().unwrap();
        if guard.is_none() {
            *guard = match self.fetch_description()? {
                Some(s) => Description::Fetched(s),
                None => Description::FetchedNone,
            };
        }
        Ok(())
    }

    /// Fetch the available resolutions but don't return it to avoid an unnecessary copy
    /// The result is store within the struct
    ///
    /// Used to asynchronously load the resolutions for later use
    pub fn load_resolutions(&self) -> error::Result<()> {
        let mut guard = self.files.lock().unwrap();
        if let Files::None = &*guard {
            match self.fetch_files() {
                Ok((files, streams)) => {
                    *guard = Files::Fetched(files, streams);
                }
                Err(err) => {
                    *guard = Files::FetchedError(err.clone());
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    fn fetch_files(&self) -> error::Result<(Vec<File>, Vec<StreamingPlaylist>)> {
        let (files, streams) = self.instance.video_complete(self.host(), &self.uuid)?;
        let files: Vec<File> = files.into_iter().map(|v| v.into()).collect();
        let streams: Vec<StreamingPlaylist> = streams.into_iter().map(|v| v.into()).collect();

        if files.is_empty() && streams.is_empty() {
            return Err(error::Error::NoContent);
        }

        Ok((files, streams))
    }

    /// Get the available resolutions
    /// During the lifetime of the struct, the resolutions will be fetched only once and the result
    /// is stored and re-used
    pub fn resolutions(&self) -> error::Result<Vec<Resolution>> {
        let mut guard = self.files.lock().unwrap();
        match &*guard {
            Files::None => match self.fetch_files() {
                Ok((files, streams)) => {
                    let resolutions = files
                        .iter()
                        .map(|file| Resolution::from_file(file))
                        .collect();
                    *guard = Files::Fetched(files, streams);
                    Ok(resolutions)
                }
                Err(err) => {
                    *guard = Files::FetchedError(err.clone());
                    Err(err)
                }
            },
            Files::FetchedError(err) => Err(err.clone()),
            Files::Fetched(files, _) => Ok(files
                .iter()
                .map(|file| Resolution::from_file(file))
                .collect()),
        }
    }

    /// Get the available streams
    /// During the lifetime of the struct, the streams will be fetched only once and the result
    /// is stored and re-used
    pub fn streams(&self) -> error::Result<Vec<StreamingPlaylist>> {
        let mut guard = self.files.lock().unwrap();
        match &*guard {
            Files::None => match self.fetch_files() {
                Ok((files, streams)) => {
                    let streams_cl = streams.clone();
                    *guard = Files::Fetched(files, streams);
                    Ok(streams_cl)
                }
                Err(err) => {
                    *guard = Files::FetchedError(err.clone());
                    Err(err)
                }
            },
            Files::FetchedError(err) => Err(err.clone()),
            Files::Fetched(_, streams) => Ok(streams.clone()),
        }
    }

    /// Get a url for a given resolution
    pub fn resolution_url(&self, id: usize) -> error::Result<String> {
        let guard = self.files.lock().unwrap();

        match &*guard {
            Files::Fetched(res, _) if res.len() > id => Ok(res[id].download_url.clone()),
            Files::Fetched(res, _) => Err(Error::OutOfBound(res.len())),
            Files::FetchedError(err) => Err(err.clone()),
            Files::None => panic!("Resolution hasn't been fetched yet"),
        }
    }

    /// Get a torrent url for a given resolution
    pub fn torrent_url(&self, id: usize) -> error::Result<String> {
        let guard = self.files.lock().unwrap();

        match &*guard {
            Files::Fetched(res, _) if res.len() > id => Ok(res[id].torrent_download_url.clone()),
            Files::Fetched(res, _) => Err(Error::OutOfBound(res.len())),
            Files::FetchedError(err) => Err(err.clone()),
            Files::None => panic!("Resolution hasn't been fetched yet"),
        }
    }

    /// Get a torrent url for a given resolution
    pub fn stream_url(&self, id: usize) -> error::Result<String> {
        let guard = self.files.lock().unwrap();

        match &*guard {
            Files::Fetched(_, res) if res.len() > id => Ok(res[id].playlist_url.clone()),
            Files::Fetched(_, res) => Err(Error::OutOfBound(res.len())),
            Files::FetchedError(err) => Err(err.clone()),
            Files::None => panic!("Resolution hasn't been fetched yet"),
        }
    }

    pub fn has_streams(&self) -> error::Result<bool> {
        let guard = self.files.lock().unwrap();
        match &*guard {
            Files::Fetched(_, streams) if !streams.is_empty() => Ok(true),
            Files::Fetched(_, _) => Ok(false),
            Files::FetchedError(err) => Err(err.clone()),
            Files::None => panic!("Resolution hasn't been fetched yet"),
        }
    }

    pub fn has_files(&self) -> error::Result<bool> {
        let guard = self.files.lock().unwrap();
        match &*guard {
            Files::Fetched(files, _) if !files.is_empty() => Ok(true),
            Files::Fetched(_, _) => Ok(false),
            Files::FetchedError(err) => Err(err.clone()),
            Files::None => panic!("Resolution hasn't been fetched yet"),
        }
    }
}
