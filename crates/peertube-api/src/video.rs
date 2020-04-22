use tokio::sync::Mutex;

use std::error;

use crate::instance::Instance;

pub struct Video<'i> {
    instance: &'i Instance,
    name: String,
    uuid: String,
    duration: u64,
    short_desc: Option<String>,
    description: Mutex<Option<Option<String>>>,
}

impl<'s> Video<'s> {
    pub fn new(
        instance: &'s Instance,
        name: String,
        uuid: String,
        duration: u64,
        short_desc: Option<String>,
    ) -> Video<'s> {
        Video {
            instance,
            name,
            uuid,
            duration,
            short_desc,
            description: Mutex::new(None),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn uuid(&self) -> &String {
        &self.uuid
    }

    pub fn short_description(&self) -> &Option<String> {
        &self.short_desc
    }

    pub fn duration(&self) -> u64 {
        self.duration
    }

    pub async fn description(&self) -> Result<Option<String>, Box<dyn error::Error>> {
        let mut guard = self.description.lock().await;
        if guard.is_none() {
            *guard = Some(self.instance.video_description(&self.uuid).await?);
        }
        Ok(guard.as_ref().unwrap().clone())
    }
}
