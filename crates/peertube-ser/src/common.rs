use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Avatar {
    pub path: String,
    pub createdAt: Option<String>,
    pub updatedAt: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub displayName: String,
    pub url: String,
    pub host: String,
    pub Avatar: Option<Avatar>,
}
