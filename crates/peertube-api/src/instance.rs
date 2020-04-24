use std::error;
#[cfg(not(feature = "send"))]
use std::rc::Rc as FeaturedRc;
#[cfg(feature = "send")]
use std::sync::Arc as FeaturedRc;

use reqwest::Client;

use peertube_ser::search::Search;
use peertube_ser::video::Description;

use crate::video::Video;

pub struct Instance {
    client: Client,
    host: String,
}

impl Instance {
    pub fn new(host: String) -> FeaturedRc<Instance> {
        FeaturedRc::new(Instance {
            client: Client::new(),
            host,
        })
    }

    pub async fn search_videos(
        self: &FeaturedRc<Instance>,
        query: &str,
    ) -> Result<Vec<Video>, Box<dyn error::Error>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/videos");
        let mut search_res: Search = serde_json::from_str(
            &*self
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
            if let Some(v) = Video::maybe_from(self, video) {
                res.push(v);
            }
        }

        Ok(res)
    }

    pub async fn video_description(
        self: &FeaturedRc<Instance>,
        uuid: &str,
    ) -> Result<Option<String>, Box<dyn error::Error>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);
        url.push_str("/description");

        let desc: Description =
            serde_json::from_str(&*self.client.get(&url).send().await?.text().await?)?;
        Ok(desc.description)
    }

    pub fn host(&self) -> &String {
        &self.host
    }
}
