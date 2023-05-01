use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tarpc::context;

/// Key-value store service
#[tarpc::service]
pub trait Node {
    /// Gets the associated value
    async fn get(key: String) -> Option<String>;
    /// Inserts key-value pair into the store
    async fn insert(key: String, value: String) -> Option<String>;
    /// Removes the key-value pair from the store
    async fn remove(key: String) -> Option<String>;
}

/// Storage layer
pub type Store = Arc<Mutex<HashMap<String, String>>>;
#[derive(Clone)]
pub struct NodeServer {
    /// Storage manager
    store: Store,
}

impl NodeServer {
    /// Creates a new key-value store node server
    pub fn new(store: Store) -> Self {
        NodeServer { store }
    }
}

#[tarpc::server]
impl Node for NodeServer {
    async fn get(self, _: context::Context, key: String) -> Option<String> {
        log::info!("Get({})", &key);
        let store = self.store.lock().unwrap();
        store.get(&key).map(|val| val.to_owned())
    }

    async fn insert(self, _: context::Context, key: String, value: String) -> Option<String> {
        log::info!("Insert({}, {})", &key, &value);
        let mut store = self.store.lock().unwrap();
        let res = store.insert(key, value);
        log::info!("{:?}", res);
        return res;
    }

    async fn remove(self, _: context::Context, key: String) -> Option<String> {
        log::info!("Remove({})", &key);
        let mut store = self.store.lock().unwrap();
        store.remove(&key)
    }
}
