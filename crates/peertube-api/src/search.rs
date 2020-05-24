use tokio::task::{spawn_local, JoinHandle};

use std::rc::Rc;

use crate::error::{self, Error};
use crate::Instance;
use crate::PreloadableList;
use crate::Video;

type Loading = JoinHandle<Result<(Vec<Video>, Option<usize>), Error>>;
pub struct VideoSearch {
    instance: Rc<Instance>,

    preload_res: bool,
    loaded: Vec<Vec<Rc<Video>>>,
    loading: Option<Loading>,
    query: String,
    current: usize,
    step: usize,
    total: Option<usize>,
}

impl VideoSearch {
    pub fn new(instance: Rc<Instance>, query: &str, step: usize) -> VideoSearch {
        VideoSearch {
            instance,
            loaded: Vec::new(),
            loading: None,
            preload_res: false,
            query: query.to_owned(),
            current: 0,
            step,
            total: None,
        }
    }
}

impl VideoSearch {
    pub async fn next_videos(&mut self) -> error::Result<&Vec<Rc<Video>>> {
        if !self.loaded.is_empty() {
            self.current += 1;
        }
        if self.loaded.len() <= self.current {
            let temp;
            if let Some(handle) = self.loading.take() {
                temp = handle.await.unwrap()?;
            } else {
                temp = self
                    .instance
                    .search_videos(&self.query, self.step, self.current * self.step)
                    .await?;
            }
            let (videos, new_total) = temp;
            self.loaded.push(videos.into_iter().map(Rc::new).collect());
            self.total = new_total.or(self.total);
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

impl PreloadableList for VideoSearch {
    type Current = Vec<Rc<Video>>;

    fn preload_next(&mut self) {
        if self.loaded.len() <= self.current + 1 && self.loading.is_none() {
            let inst_cloned = self.instance.clone();
            let quer_cloned = self.query.clone();
            let nb = self.step;
            let skip = (self.current + 1) * self.step;
            self.loading = Some(spawn_local(async move {
                inst_cloned.search_videos(&quer_cloned, nb, skip).await
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

    fn offset(&self) -> usize {
        self.current * self.step
    }

    fn expected_total(&self) -> Option<usize> {
        self.total
    }
}