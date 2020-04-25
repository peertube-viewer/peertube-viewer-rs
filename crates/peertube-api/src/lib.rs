extern crate chrono;
extern crate peertube_ser;
extern crate reqwest;
extern crate serde_json;
extern crate tokio;

mod instance;
mod video;

pub use instance::Instance;
pub use video::{Resolution, Video};
