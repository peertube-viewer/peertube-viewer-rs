use crate::instance::Instance;
pub struct Video<'i> {
    instance: &'i Instance,
    name: String,
    uuid: String,
}

impl<'s> Video<'s> {
    pub fn new(instance: &'s Instance, name: String, uuid: String) -> Video<'s> {
        Video {
            instance,
            name,
            uuid,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn uuid(&self) -> &String {
        &self.uuid
    }
}
