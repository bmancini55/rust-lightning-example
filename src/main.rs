use bitcoin::network::constants::Network;
use bitcoin::secp256k1::{PublicKey, SecretKey, Secp256k1};
use std::net::SocketAddr;
use tokio;

#[macro_use]
mod log_macros;

mod chain;
mod channelpersistor;
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
    let node_key_str = "961cbdc16536c7df1e2ba6006e3e9c8f6dd2446850c207f25276a7b355fc60a5";
    let node_key_slice = hex::decode(node_key_str).unwrap();
    let node_key = SecretKey::from_slice(&node_key_slice).unwrap();

    // construct and log the pubkey
    let secp = Secp256k1::new();
    let node_pubkey = PublicKey::from_secret_key(&secp, &node_key);
    log_info!(logger.clone(), "using pubkey {}", log_pubkey!(&node_pubkey));

    let user_config = lightning::util::config::UserConfig::default();


    // construct hte demo client
    let demo_client = Arc::new(client::LightingClient::new(node_key, &seed, user_config, Network::Testnet, logger.clone()));

    // lightning
    // let node_id_str = "02462af1452a7c81b9f448e8137786b520bec9d15f3d864acca3f6672936e492ff";
    // let node_addr_str = "192.168.0.113:9735";

    // demo2.lndexplorer.com (LND v0.11)
    let node_id_str = "03b1cf5623ca6757d49de3b6e2b9340065ba991c75b8e9cd8aec51dc54322cbd1d";
    let node_addr_str = "38.87.54.164:9745";

    // lnbig1
    // let node_id_str = "02312627fdf07fbdd7e5ddb136611bdde9b00d26821d14d94891395452f67af248";
    // let node_addr_str = "23.237.77.12:9735";

    // demo1.lndexplorer.com (LNV v0.7)
    // let node_id_str = "036b96e4713c5f84dcb8030592e1bd42a2d9a43d91fa2e535b9bfd05f2c5def9b9";
    // let node_addr_str = "38.87.54.163:9745";

    // eclair
    // let node_id_str = "03933884aaf1d6b108397e5efe5c86bcf2d8ca8d2f700eda99db9214fc2712b134";
    // let node_addr_str = "34.250.234.192:9735";


    // connect to the remote node
    let demo_client_clone = demo_client.clone();
    tokio::spawn(async move {
        let node_id_slice = &hex::decode(node_id_str).unwrap();
        let node_id = PublicKey::from_slice(&node_id_slice).unwrap();
        let node_addr: SocketAddr = node_addr_str.parse().unwrap();
        demo_client_clone.connect_to_node(node_id, node_addr).await;
    });


    // fire up the server
    tokio::spawn(async move {
        demo_client.listen("127.0.0.1:9736").await.unwrap();
    }).await.unwrap();
}
