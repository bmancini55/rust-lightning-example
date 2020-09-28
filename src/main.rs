use bitcoin::network::constants::Network;
use bitcoin::secp256k1::key::{PublicKey, SecretKey};
use std::net::SocketAddr;
use tokio;

#[macro_use]
mod log_macros;

mod chain;
mod client;
mod log;

use std::sync::Arc;

type Logger = dyn lightning::util::logger::Logger;

#[tokio::main]
async fn main() {
    // construct a concrete logger for our application that will
    // simply log to the console. not much special there.
    let logger: Arc<Logger> = Arc::new(log::ConsoleLogger::new());

    let seed = rand::random::<[u8; 32]>();
    let node_key_str = "d84985781fee4676a616f81399d28cced95a691a983c582b6285108e02830673";
    let node_key_slice = hex::decode(node_key_str).unwrap();
    let node_key = SecretKey::from_slice(&node_key_slice).unwrap();

    let user_config = lightning::util::config::UserConfig::default();


    // construct hte demo client
    let demo_client = Arc::new(client::LightingClient::new(node_key, &seed, user_config, Network::Testnet, logger.clone()));

    // demo2.lndexplorer.com (LND v0.11)
    let node_id_str = "03b1cf5623ca6757d49de3b6e2b9340065ba991c75b8e9cd8aec51dc54322cbd1d";
    let node_addr_str = "38.87.54.164:9745";

    // demo1.lndexplorer.com (LNV v0.7)
    // let node_id_str = "036b96e4713c5f84dcb8030592e1bd42a2d9a43d91fa2e535b9bfd05f2c5def9b9";
    // let node_addr_str = "38.87.54.163:9745";

    // eclair
    // let node_id_str = "03933884aaf1d6b108397e5efe5c86bcf2d8ca8d2f700eda99db9214fc2712b134";
    // let node_addr_str = "34.250.234.192:9735";

    let node_id_slice = &hex::decode(node_id_str).unwrap();
    let node_id = PublicKey::from_slice(&node_id_slice).unwrap();
    let node_addr: SocketAddr = node_addr_str.parse().unwrap();
    demo_client.connect_to_node(node_id, node_addr).await;
}
