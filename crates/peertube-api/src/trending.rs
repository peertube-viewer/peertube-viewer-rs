// This file is part of peertube-viewer-rs.
// 
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
// 
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>. 


use tokio::task::{spawn_local, JoinHandle};

use std::sync::Arc;

use crate::error::{self, Error};
use crate::Instance;
use crate::PreloadableList;
use crate::Video;

type Loading = JoinHandle<Result<(Vec<Video>, Option<usize>), Error>>;
pub struct TrendingList {
    instance: Arc<Instance>,

    preload_res: bool,
    loaded: Vec<Vec<Arc<Video>>>,
    loading: Option<Loading>,
    current: usize,
    step: usize,
    total: Option<usize>,
}

impl TrendingList {
    pub fn new(instance: Arc<Instance>, step: usize) -> TrendingList {
        TrendingList {
            instance,
            loaded: Vec::new(),
            loading: None,
            preload_res: false,
            current: 0,
            step,
            total: None,
        }
    }
}

impl TrendingList {
    pub async fn next_videos(&mut self) -> error::Result<&Vec<Arc<Video>>> {
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
                    .trending_videos(self.step, self.current * self.step)
                    .await?;
            }
            let (videos, new_total) = temp;
            self.loaded.push(videos.into_iter().map(Arc::new).collect());
            self.total = new_total.or(self.total);
        }
        Ok(&self.loaded[self.current])
    }

    pub fn preload_res(&mut self, should: bool) {
        self.preload_res = should;
    }

    pub fn prev(&mut self) -> &Vec<Arc<Video>> {
        self.current -= 1;
        &self.loaded[self.current]
    }
}

impl PreloadableList for TrendingList {
    type Current = Vec<Arc<Video>>;

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

    fn current(&self) -> &Vec<Arc<Video>> {
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
