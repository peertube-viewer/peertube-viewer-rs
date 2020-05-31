use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Avatar {
    pub path: String,
    pub createdAt: Option<String>,
    pub updatedAt: Option<String>,
}
