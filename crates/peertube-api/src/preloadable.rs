mod channels;
mod videos;

pub use channels::ChannelSearch;
pub use videos::VideoList;

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
