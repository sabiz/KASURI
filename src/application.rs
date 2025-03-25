
#[derive(Clone, Debug)]
pub struct Application {
    pub name: String,
}

impl Application {
    pub fn new(name: String) -> Self {
        Self {
            name
        }
    }
}