use std::collections::HashMap;

pub struct Environment {
    variable: HashMap<String, i32>,
}
impl Environment {
    pub fn new() -> Self {
        Self {
            variable: HashMap::new(),
        }
    }
    pub fn append_entry(&mut self, key: String, pos: i32) {
        self.variable.insert(key, pos);
    }
    pub fn get_entry(&self, key: String) -> Option<i32> {
        if let Some(v) = self.variable.get(&key) {
            Some(*v)
        } else {
            None
        }
    }
}
