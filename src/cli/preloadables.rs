// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use peertube_api::{channels::Channel, error, Comment, Instance, Video};
use peertube_viewer_utils::host_from_handle;
use preloadable_list::AsyncLoader;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::spawn;

#[derive(Clone)]
enum VideoMode {
    Search(String),
    Channel(String),
    Trending,
}

pub struct Videos {
    instance: Arc<Instance>,
    mode: VideoMode,
    preload_res: AtomicBool,
}

impl Videos {
    pub fn new_search(instance: Arc<Instance>, query: &str) -> Videos {
        Videos {
            instance,
            preload_res: AtomicBool::new(false),
            mode: VideoMode::Search(query.to_owned()),
        }
    }

    pub fn new_channel(instance: Arc<Instance>, handle: &str) -> Videos {
        Videos {
            instance,
            preload_res: AtomicBool::new(false),
            mode: VideoMode::Channel(handle.to_owned()),
        }
    }

    pub fn new_trending(instance: Arc<Instance>) -> Videos {
        Videos {
            instance,
            preload_res: AtomicBool::new(false),
            mode: VideoMode::Trending,
        }
    }

    pub fn preload_res(&self, should: bool) {
        self.preload_res.store(should, Ordering::SeqCst);
    }

    pub fn name(&self) -> &'static str {
        match self.mode {
            VideoMode::Search(_) => "Video search",
            VideoMode::Trending => "Trending video",
            VideoMode::Channel(_) => "Channel videos",
        }
    }
}

impl AsyncLoader for Videos {
    type Data = Video;
    type Error = error::Error;

    fn data(&self, step: usize, offset: usize) -> Result<(Vec<Self::Data>, usize), Self::Error> {
        match &self.mode {
            VideoMode::Search(query) => self.instance.search_videos(query, step, offset),
            VideoMode::Channel(handle) => self.instance.channel_videos(
                host_from_handle(handle)
                    .as_ref()
                    .map(|s| &**s)
                    .unwrap_or(""),
                handle,
                step,
                offset,
            ),
            VideoMode::Trending => self.instance.trending_videos(step, offset),
        }
    }

    fn item(&self, vid: Arc<Video>) {
        if self.preload_res.load(Ordering::SeqCst) {
            let cl2 = vid.clone();
            #[allow(unused_must_use)]
            spawn(move || cl2.load_resolutions());
        }
        spawn(move || vid.load_description());
    }
}

pub struct Channels {
    instance: Arc<Instance>,
    query: String,
}

impl Channels {
    pub fn new(instance: Arc<Instance>, query: &str) -> Channels {
        Channels {
            instance,
            query: query.to_owned(),
        }
    }
}

pub struct Comments {
    instance: Arc<Instance>,
    video_uuid: String,
    host: String,
}

impl AsyncLoader for Channels {
    type Data = Channel;
    type Error = error::Error;

    fn data(&self, step: usize, offset: usize) -> Result<(Vec<Channel>, usize), error::Error> {
        self.instance.search_channels(&self.query, step, offset)
    }
}

impl Comments {
    pub fn new(instance: Arc<Instance>, host: String, video_uuid: String) -> Comments {
        Comments {
            instance,
            video_uuid,
            host,
        }
    }
}

impl AsyncLoader for Comments {
    type Data = Comment;
    type Error = error::Error;

    fn data(&self, step: usize, offset: usize) -> Result<(Vec<Comment>, usize), error::Error> {
        self.instance
            .comments(&self.host, &self.video_uuid, step, offset)
    }
}
