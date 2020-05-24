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
    type Current;

    fn preload_next(&mut self) {}
    fn preload_prev(&mut self) {}

    fn current_len(&self) -> usize;
    fn current(&self) -> &Self::Current;

    fn offset(&self) -> usize;
    fn expected_total(&self) -> Option<usize> {
        None
    }

    #[allow(unused)]
    fn preload_id(&mut self, id: usize) {}
}
