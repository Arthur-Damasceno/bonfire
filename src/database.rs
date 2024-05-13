use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

type Map = HashMap<Vec<u8>, Vec<u8>>;

#[derive(Clone, Default)]
pub struct Database(Arc<RwLock<Map>>);

impl Database {
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.read().unwrap().get(key).map(|x| x.clone())
    }

    pub fn set(&self, key: Vec<u8>, value: Vec<u8>) {
        self.0.write().unwrap().insert(key, value);
    }

    pub fn delete(&self, key: &[u8]) -> bool {
        self.0.write().unwrap().remove(key).is_some()
    }
}
