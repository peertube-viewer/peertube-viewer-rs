use tokio::task::{spawn_local, JoinHandle};

use std::rc::Rc;

use crate::channels::Channel;
use crate::error::{self, Error};
use crate::Instance;
use crate::PreloadableList;

type Loading = JoinHandle<Result<(Vec<Channel>, Option<usize>), Error>>;
pub struct ChannelSearch {
    instance: Rc<Instance>,

    loaded: Vec<Vec<Rc<Channel>>>,
    loading: Option<Loading>,
    query: String,
    current: usize,
    step: usize,
    total: Option<usize>,
}

impl ChannelSearch {
    pub fn new(instance: Rc<Instance>, query: &str, step: usize) -> ChannelSearch {
        ChannelSearch {
            instance,
            loaded: Vec::new(),
            loading: None,
            query: query.to_owned(),
            current: 0,
            step,
            total: None,
        }
    }
}

impl ChannelSearch {
    pub async fn next_channels(&mut self) -> error::Result<&Vec<Rc<Channel>>> {
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
                    .search_channels(&self.query, self.step, self.current * self.step)
                    .await?;
            }
            let (channels, new_total) = temp;
            self.loaded
                .push(channels.into_iter().map(Rc::new).collect());
            self.total = new_total.or(self.total);
        }
        Ok(&self.loaded[self.current])
    }

    pub fn prev(&mut self) -> &Vec<Rc<Channel>> {
        self.current -= 1;
        &self.loaded[self.current]
    }
}

impl PreloadableList for ChannelSearch {
    type Current = Vec<Rc<Channel>>;

    fn preload_next(&mut self) {
        if self.loaded.len() <= self.current + 1 && self.loading.is_none() {
            let inst_cloned = self.instance.clone();
            let quer_cloned = self.query.clone();
            let nb = self.step;
            let skip = (self.current + 1) * self.step;
            self.loading = Some(spawn_local(async move {
                inst_cloned.search_channels(&quer_cloned, nb, skip).await
            }));
        }
    }

    fn current(&self) -> &Vec<Rc<Channel>> {
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
