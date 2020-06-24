use peertube_ser::common;
use peertube_ser::search;

#[derive(Clone, Debug)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub url: String,
    pub host: String,
}

#[allow(unused)]
impl Channel {
    fn id(&self) -> &i64 {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn display_name(&self) -> &str {
        &self.display_name
    }
    fn url(&self) -> &str {
        &self.url
    }
    fn host(&self) -> &str {
        &self.host
    }
}

impl From<common::Channel> for Channel {
    fn from(c: common::Channel) -> Channel {
        Channel {
            id: c.id,
            name: c.name,
            display_name: c.displayName,
            url: c.url,
            host: c.host,
        }
    }
}
