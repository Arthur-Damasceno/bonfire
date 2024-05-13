use std::{collections::HashMap, sync::RwLock};

use tokio::sync::broadcast::{channel, Receiver, Sender};

type Data = HashMap<Vec<u8>, Vec<u8>>;
type Channels = HashMap<u32, Sender<Vec<u8>>>;

#[derive(Default)]
pub struct Database {
    data: RwLock<Data>,
    channels: RwLock<Channels>,
}

impl Database {
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.read().unwrap().get(key).map(|x| x.clone())
    }

    pub fn set(&self, key: Vec<u8>, value: Vec<u8>) {
        self.data.write().unwrap().insert(key, value);
    }

    pub fn delete(&self, key: &[u8]) -> bool {
        self.data.write().unwrap().remove(key).is_some()
    }

    pub fn publish(&self, id: u32, message: Vec<u8>) -> bool {
        if let Some(sender) = self.channels.read().unwrap().get(&id) {
            sender.send(message).is_ok()
        } else {
            false
        }
    }

    pub fn subscribe(&self, id: u32) -> Receiver<Vec<u8>> {
        if let Some(tx) = self.channels.read().unwrap().get(&id) {
            tx.subscribe()
        } else {
            let (tx, rx) = channel(1_000);

            self.channels.write().unwrap().insert(id, tx);

            rx
        }
    }

    pub fn unsubscribe(&self, id: u32) {
        if let Some(tx) = self.channels.read().unwrap().get(&id) {
            if tx.receiver_count() == 0 {
                self.channels.write().unwrap().remove(&id);
            }
        }
    }
}
