use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr},
    sync::Arc,
    sync::Mutex,
    time::Duration,
};

use log;

use anyhow::Result;
use clap::Parser;
use dht::HashNode;
use futures::{future, prelude::*};
use rand::{distributions::Uniform, prelude::Distribution, thread_rng};
use tarpc::{
    context,
    server::{self, Channel},
};
use tarpc::{server::incoming::Incoming, tokio_serde::formats::Json};
use tokio::time;

#[derive(Debug, Parser)]
struct Flags {
    /// listening port
    #[clap(long)]
    port: u16,
}

#[derive(Clone)]
struct HashNodeServer {
    store: HashMap<String, String>,
}

impl HashNodeServer {
    pub fn new(store: HashMap<String, String>) -> Self {
        HashNodeServer { store }
    }
}

#[tarpc::server]
impl HashNode for HashNodeServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        let sleep_time =
            Duration::from_millis(Uniform::new_inclusive(1, 10).sample(&mut thread_rng()));
        time::sleep(sleep_time).await;
        format!("Hello, {}!", name)
    }

    async fn get(self, _: context::Context, key: String) -> Option<String> {
        log::info!("Get({})", &key);
        self.store.get(&key).map(|val| val.to_owned())
    }

    async fn insert(mut self, _: context::Context, key: String, value: String) -> Option<String> {
        log::info!("Insert({}, {})", &key, &value);
        let res = self.store.insert(key, value);
        log::info!("{:?}", res);
        return res;
    }

    async fn remove(mut self, _: context::Context, key: String) -> Option<String> {
        log::info!("Remove({})", &key);
        self.store.remove(&key)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting up");
    let flags = Flags::parse();
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), flags.port);
    let server = Arc::new(Mutex::new(HashNodeServer::new(HashMap::new())));
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    let handle = tokio::spawn(async move {
        listener.config_mut().max_frame_length(usize::MAX);
        listener
            // Ignore accept errors.
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            // Limit channels to 1 per IP.
            .max_channels_per_key(2, |t| t.transport().peer_addr().unwrap().ip())
            .map(|channel| {
                let server = Arc::clone(&server);
                channel.execute({
                    move || {
                        let server = server.lock().unwrap();
                        server.serve()
                    }
                })
            })
            // Max 10 channels.
            .buffer_unordered(10)
            .for_each(|_| async {})
            .await;
    });

    handle.await;

    Ok(())
}
