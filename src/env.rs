use std::collections::LinkedList;

#[derive(Clone, Debug)]
pub struct Environment {
    variables: LinkedList<(String, i64)>,
}
impl Environment {
    pub fn new() -> Self {
        Self {
            variables: LinkedList::new(),
        }
    }
    pub fn bind(&mut self, key: String, pos: i64) {
        self.variables.push_back((key, pos));
    }
    pub fn lookup(&self, key: &str) -> Option<i64> {
        self.variables
            .iter()
            .rev()
            .filter(|(k, _)| k == key)
            .next()
            .map(|(_, v)| *v)
    }
}
