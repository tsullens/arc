use std::collections::HashMap;
use std::sync::{RwLock};

#[derive(Debug)]
pub struct Database {
     db: RwLock<HashMap<String, String>>,
}

impl Database {
    pub fn init() -> Self {
        let map = HashMap::new();
        Database { db: RwLock::new(map) }
    }
}