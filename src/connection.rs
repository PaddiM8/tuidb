pub struct Connection {
    pub name: String,
}

impl Connection {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
