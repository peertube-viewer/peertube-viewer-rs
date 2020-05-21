extern crate chrono;
extern crate peertube_ser;
extern crate reqwest;
extern crate serde_json;
extern crate tokio;

pub mod error;
mod instance;
mod search;
mod video;

pub use instance::Instance;
pub use search::VideoSearch;
pub use video::{Resolution, Video};

use tokio::task::JoinHandle;

pub trait PreloadableList {
    fn preload_next(&mut self) {}
    fn preload_prev(&mut self) {}

    fn len(&self) -> usize;

    #[allow(unused)]
    fn preload_id(&mut self, id: usize) {}
}
