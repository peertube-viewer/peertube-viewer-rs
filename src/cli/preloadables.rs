use peertube_api::{channels::Channel, error, Instance, Video};
use preloadable_list::AsyncLoader;
use std::{future::Future, pin::Pin, rc::Rc};
use tokio::task::spawn_local;

#[derive(Clone)]
enum VideoMode {
    Search(String),
    Channel(String),
    Trending,
}

pub struct Videos {
    instance: Rc<Instance>,
    mode: VideoMode,
    preload_res: bool,
}

impl Videos {
    pub fn new_search(instance: Rc<Instance>, query: &str) -> Videos {
        Videos {
            instance,
            preload_res: false,
            mode: VideoMode::Search(query.to_owned()),
        }
    }

    pub fn new_channel(instance: Rc<Instance>, handle: &str) -> Videos {
        Videos {
            instance,
            preload_res: false,
            mode: VideoMode::Channel(handle.to_owned()),
        }
    }

    pub fn new_trending(instance: Rc<Instance>) -> Videos {
        Videos {
            instance,
            preload_res: false,
            mode: VideoMode::Trending,
        }
    }

    pub fn preload_res(&mut self, should: bool) {
        self.preload_res = should;
    }
}

impl AsyncLoader for Videos {
    type Data = Video;
    type Error = error::Error;

    fn data(
        &mut self,
        step: usize,
        offset: usize,
    ) -> Pin<Box<dyn 'static + Future<Output = Result<(Vec<Video>, Option<usize>), error::Error>>>>
    {
        let instance_cl = self.instance.clone();
        let mode_cl = self.mode.clone();
        Box::pin(async move {
            match mode_cl {
                VideoMode::Search(query) => instance_cl.search_videos(&query, step, offset).await,
                VideoMode::Channel(handle) => {
                    instance_cl.channel_videos(&handle, step, offset).await
                }
                VideoMode::Trending => instance_cl.trending_videos(step, offset).await,
            }
        })
    }

    fn item(&self, vid: Rc<Video>) {
        if self.preload_res {
            let cl2 = vid.clone();
            #[allow(unused_must_use)]
            spawn_local(async move {
                cl2.load_resolutions().await;
            });
        }
        spawn_local(async move { vid.load_description().await });
    }
}

pub struct Channels {
    instance: Rc<Instance>,
    query: String,
}

impl Channels {
    pub fn new(instance: Rc<Instance>, query: &str) -> Channels {
        Channels {
            instance,
            query: query.to_owned(),
        }
    }
}

impl AsyncLoader for Channels {
    type Data = Channel;
    type Error = error::Error;

    fn data(
        &mut self,
        step: usize,
        offset: usize,
    ) -> Pin<Box<dyn 'static + Future<Output = Result<(Vec<Channel>, Option<usize>), error::Error>>>>
    {
        let instance_cl = self.instance.clone();
        let query_cl = self.query.clone();
        Box::pin(async move { instance_cl.search_channels(&query_cl, step, offset).await })
    }
}
