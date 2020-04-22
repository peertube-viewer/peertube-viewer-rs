use std::error;

use reqwest::{Client, Error, Method};
use serde_json;

use peertube_ser::search::Search;

use crate::video::Video;

pub struct Instance {
    client: Client,
    host: String,
}

impl Instance {
    pub fn new(host: String) -> Instance {
        Instance {
            client: Client::new(),
            host,
        }
    }

    pub async fn search_videos<'s>(
        &'s self,
        query: &str,
    ) -> Result<Vec<Video<'s>>, Box<dyn error::Error>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/videos");
        let mut search_res: Search = serde_json::from_str(
            &self
                .client
                .get(&url)
                .query(&[("search", query)])
                .send()
                .await?
                .text()
                .await?,
        )?;
        let mut res = Vec::new();
        for video in search_res.data.drain(..) {
            if let (Some(name), Some(uuid)) = (video.name, video.uuid) {
                res.push(Video::new(&self, name, uuid));
            }
        }

        Ok(res)
    }
}
