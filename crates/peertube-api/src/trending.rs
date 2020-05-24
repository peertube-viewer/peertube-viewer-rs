use tokio::task::{spawn_local, JoinHandle};

use std::rc::Rc;

use crate::error::{self, Error};
use crate::Instance;
use crate::PreloadableList;
use crate::Video;

pub struct TrendingList {
    instance: Rc<Instance>,

    preload_res: bool,
    loaded: Vec<Vec<Rc<Video>>>,
    loading: Option<JoinHandle<Result<Vec<Video>, Error>>>,
    current: usize,
    step: usize,
}

impl TrendingList {
    pub fn new(instance: Rc<Instance>, step: usize) -> TrendingList {
        TrendingList {
            instance,
            loaded: Vec::new(),
            loading: None,
            preload_res: false,
            current: 0,
            step,
        }
    }
}

impl TrendingList {
    pub async fn next_videos(&mut self) -> error::Result<&Vec<Rc<Video>>> {
        if !self.loaded.is_empty() {
            self.current += 1;
        }
        if self.loaded.len() <= self.current {
            if let Some(handle) = self.loading.take() {
                self.loaded
                    .push(handle.await.unwrap()?.into_iter().map(Rc::new).collect())
            } else {
                self.loaded.push(
                    self.instance
                        .trending_videos(self.step, self.current * self.step)
                        .await?
                        .into_iter()
                        .map(Rc::new)
                        .collect(),
                );
            }
        }
        Ok(&self.loaded[self.current])
    }

    pub fn preload_res(&mut self, should: bool) {
        self.preload_res = should;
    }

    pub fn prev(&mut self) -> &Vec<Rc<Video>> {
        self.current -= 1;
        &self.loaded[self.current]
    }
}

impl PreloadableList for TrendingList {
    type Current = Vec<Rc<Video>>;

    fn preload_next(&mut self) {
        if self.loaded.len() <= self.current + 1 && self.loading.is_none() {
            let inst_cloned = self.instance.clone();
            let nb = self.step;
            let skip = (self.current + 1) * self.step;
            self.loading = Some(spawn_local(async move {
                inst_cloned.trending_videos(nb, skip).await
            }));
        }
    }

    fn preload_id(&mut self, id: usize) {
        let video_cloned = self.current()[id].clone();
        spawn_local(async move { video_cloned.load_description().await });
        if self.preload_res {
            let cl2 = self.current()[id].clone();
            #[allow(unused_must_use)]
            spawn_local(async move {
                cl2.load_resolutions().await;
            });
        }
    }

    fn current(&self) -> &Vec<Rc<Video>> {
        &self.loaded[self.current]
    }

    fn current_len(&self) -> usize {
        self.current().len()
    }
}
