// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use peertube_ser::common;

#[derive(Clone, Debug)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub url: Option<String>,
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
    fn url(&self) -> &Option<String> {
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
