use std::error;

use reqwest::Client;
use serde_json;

use peertube_ser::search::Search;
use peertube_ser::video::Description;

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

    pub async fn search_videos(
        &self,
        query: &str,
    ) -> Result<Vec<Video<'_>>, Box<dyn error::Error>> {
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
            if let (Some(name), Some(uuid), Some(mut duration)) =
                (video.name, video.uuid, video.duration)
            {
                if duration < 0 {
                    duration = 0;
                };
                res.push(Video::new(
                    &self,
                    name,
                    uuid,
                    duration as u64,
                    video.publishedAt.unwrap_or_default(),
                    video.description,
                ));
            }
        }

        Ok(res)
    }

    pub async fn video_description(
        &self,
        uuid: &str,
    ) -> Result<Option<String>, Box<dyn error::Error>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);
        url.push_str("/description");

        let desc: Description =
            serde_json::from_str(&self.client.get(&url).send().await?.text().await?)?;
        Ok(desc.description)
    }

    pub fn host(&self) -> &String {
        &self.host
    }
}
