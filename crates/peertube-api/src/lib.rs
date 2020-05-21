extern crate chrono;
extern crate peertube_ser;
extern crate reqwest;
extern crate serde_json;
extern crate tokio;

pub mod error;
mod instance;
mod search;
mod trending;
mod video;

pub use instance::Instance;
pub use search::VideoSearch;
pub use trending::TrendingList;
pub use video::{Resolution, Video};

pub trait PreloadableList {
    fn preload_next(&mut self) {}
    fn preload_prev(&mut self) {}

    fn current_len(&self) -> usize;

    #[allow(unused)]
    fn preload_id(&mut self, id: usize) {}
}
