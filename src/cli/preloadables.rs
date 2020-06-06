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
    step: usize,
    mode: VideoMode,
    preload_res: bool,
}

impl Videos {
    pub fn new_search(instance: Rc<Instance>, query: &str, step: usize) -> Videos {
        Videos {
            instance,
            preload_res: false,
            mode: VideoMode::Search(query.to_owned()),
            step,
        }
    }

    pub fn new_channel(instance: Rc<Instance>, handle: &str, step: usize) -> Videos {
        Videos {
            instance,
            preload_res: false,
            mode: VideoMode::Channel(handle.to_owned()),
            step,
        }
    }

    pub fn new_trending(instance: Rc<Instance>, step: usize) -> Videos {
        Videos {
            instance,
            preload_res: false,
            mode: VideoMode::Trending,
            step,
        }
    }
}

impl AsyncLoader for Videos {
    type Data = Video;
    type Error = error::Error;

    fn data(
        &mut self,
        current: usize,
    ) -> Pin<Box<dyn 'static + Future<Output = Result<(Vec<Video>, Option<usize>), error::Error>>>>
    {
        async fn inner(
            instance: Rc<Instance>,
            mode: VideoMode,
            current: usize,
            step: usize,
        ) -> Result<(Vec<Video>, Option<usize>), error::Error> {
            match mode {
                VideoMode::Search(query) => {
                    instance.search_videos(&query, step, current * step).await
                }
                VideoMode::Channel(handle) => {
                    instance.channel_videos(&handle, step, current * step).await
                }
                VideoMode::Trending => instance.trending_videos(step, current * step).await,
            }
        }

        let instance_cl = self.instance.clone();
        let mode_cl = self.mode.clone();
        let step = self.step;
        Box::pin(async move { inner(instance_cl, mode_cl, current, step).await })
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
    step: usize,
    query: String,
}

impl Channels {
    pub fn new(instance: Rc<Instance>, query: &str, step: usize) -> Channels {
        Channels {
            instance,
            query: query.to_owned(),
            step,
        }
    }
}

impl AsyncLoader for Channels {
    type Data = Channel;
    type Error = error::Error;

    fn data(
        &mut self,
        current: usize,
    ) -> Pin<Box<dyn 'static + Future<Output = Result<(Vec<Channel>, Option<usize>), error::Error>>>>
    {
        async fn inner(
            instance: Rc<Instance>,
            query: String,
            current: usize,
            step: usize,
        ) -> Result<(Vec<Channel>, Option<usize>), error::Error> {
            instance.search_channels(&query, step, current * step).await
        }

        let instance_cl = self.instance.clone();
        let step = self.step;
        let query_cl = self.query.clone();
        Box::pin(async move { inner(instance_cl, query_cl, current, step).await })
    }
}
