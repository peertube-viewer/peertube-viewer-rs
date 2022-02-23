// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Avatar {
    pub path: String,
    pub createdAt: Option<String>,
    pub updatedAt: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub displayName: String,

    #[serde(default)]
    pub url: Option<String>,
    pub host: String,
    pub Avatar: Option<Avatar>,
}

/// Structure used to deserialize the state of a video
#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct VideoState {
    pub id: u16,
    pub label: String,
}

pub mod dates_deser {
    use serde::{de::Error, Deserialize, Deserializer};
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<OffsetDateTime, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        OffsetDateTime::parse(&time, &Rfc3339).map_err(D::Error::custom)
    }
}
