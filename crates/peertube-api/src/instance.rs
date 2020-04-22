use std::error;

use reqwest::{Client, Error, Method};
use serde_json;

use peertube_ser::search::Search;

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

    pub async fn search_videos(&self, query: &str) -> Result<Search, Box<dyn error::Error>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/videos");
        Ok(serde_json::from_str(
            &self
                .client
                .get(&url)
                .query(&[("search", query)])
                .send()
                .await?
                .text()
                .await?,
        )?)
    }
}
