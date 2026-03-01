#[derive(Debug, Clone)]
pub struct Script {
    pub id: String,
    pub name: String,
}

impl Script {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}
