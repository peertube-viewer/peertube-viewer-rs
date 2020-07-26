#![allow(clippy::redundant_pattern_matching)] // Suppress warnings caused by nanoserde

use nanoserde::DeJson;

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Avatar {
    pub path: String,
    pub createdAt: Option<String>,
    pub updatedAt: Option<String>,
}

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub displayName: String,
    pub url: String,
    pub host: String,
    pub Avatar: Option<Avatar>,
}
