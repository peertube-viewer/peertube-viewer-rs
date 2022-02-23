// This file is part of peertube-viewer-rs.
// 
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
// 
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>. 


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
