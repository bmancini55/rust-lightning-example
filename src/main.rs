use bitcoin::network::constants::Network;
use bitcoin::secp256k1::key::SecretKey;
use std::sync::Arc;
mod chain;
mod log;

#[tokio::main]
async fn main() {
    let seed = rand::random::<[u8; 32]>();
    let node_key_str = "d84985781fee4676a616f81399d28cced95a691a983c582b6285108e02830673";
    let node_key_slice = hex::decode(node_key_str).unwrap();
    let node_key = SecretKey::from_slice(&node_key_slice).unwrap();

    let user_config = lightning::util::config::UserConfig::default();

    let democlient = Arc::new(client::LightingClient::new(
        node_key,
        &seed,
        user_config,
        Network::Testnet,
    ));

    client::connect(democlient).await;
}

mod client {
    use crate::chain;
    use crate::log;
    use bitcoin::network::constants::Network;
    use bitcoin::secp256k1::key::{PublicKey, SecretKey};
    use hex;
    use lightning::chain::keysinterface::KeysManager;
    use lightning::util::events::EventsProvider;
    use lightning::util::logger::Logger as LoggerTrait;
    use lightning::util::logger::{Level, Record};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    /// Super basic wrapper class that is going to implement the bare
    /// minimum needed so we can look for p2p messaging traffic and
    /// connect to a remote node.
    pub struct LightingClient {
        channel_manager: ChannelManager,
        channel_monitor: Arc<ChannelMonitor>,
        peer_manager: PeerManager,
    }

    impl LightingClient {
        pub fn new(
            node_key: SecretKey,
            seed: &[u8; 32],
            user_config: lightning::util::config::UserConfig,
            network: Network,
        ) -> Self {
            // construct a concrete logger for our application that will
            // simply log to the console. not much special there.
            let logger = Arc::new(log::ConsoleLogger::new());

            // constructs a bitcoin_client which implements ChainMonitor
            // TransactionBroadcaster and FeeEstimator. Since this is
            // a dev environment we'll make this a concrete type.
            let bitcoin_client = Arc::new(chain::FakeBitcoinClient::new(logger.clone()));

            // next we will construct a Arc<SimpleManyChannelMonitor>
            // that uses a ChainMonitor, FeeEstimator, TxBroadcaster,
            // and Logger.
            let channel_monitor = Arc::new(ChannelMonitor::new(
                bitcoin_client.clone(),
                bitcoin_client.clone(),
                logger.clone(),
                bitcoin_client.clone(),
            ));

            // next we construct a keys_manager from our supplied seed
            // for the appropriate network. Again we don't really need
            // to do much with this since we're only concerned with
            // gossip traffic
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            let keys_manager = Arc::new(KeysManager::new(
                &seed,
                network,
                ts.as_secs(),
                ts.subsec_nanos(),
            ));

            // next we construct a channel manager for
            let last_height: usize = 1000000;
            let channel_manager: ChannelManager =
                Arc::new(lightning::ln::channelmanager::ChannelManager::new(
                    Network::Testnet,
                    bitcoin_client.clone(),
                    channel_monitor.clone(),
                    bitcoin_client.clone(),
                    logger.clone(),
                    keys_manager.clone(),
                    user_config,
                    last_height,
                ));

            // next construct the NetGraphMsgHandler. This type will be
            // used as the RoutingMessageHandler which gets attached to
            // a MessageHandler, which is itself used in the
            // PeerManager. Cool.
            let net_graph_manager: NetGraphManager =
                Arc::new(lightning::routing::network_graph::NetGraphMsgHandler::new(
                    bitcoin_client.clone(),
                    logger.clone(),
                ));

            // Now that we have a ChannelMessageHandler (channel_manager)
            // and a RoutingMessageHandler (net_graph_manager) we can
            // we can construct a MessageHandler which contains
            // references and provides access by a Peer.
            let message_handler = lightning::ln::peer_handler::MessageHandler {
                chan_handler: channel_manager.clone(),
                route_handler: net_graph_manager.clone(),
            };

            // Now that we have the Message Handler constructed we can
            // make our PeerManager and supply it with MessageHandler
            // and some other stuff
            let peer_manager: PeerManager =
                Arc::new(lightning::ln::peer_handler::PeerManager::new(
                    message_handler,
                    node_key,
                    &rand::random::<[u8; 32]>(),
                    logger.clone(),
                ));

            // Finally we capture all this jazz in our client object
            let res = LightingClient {
                peer_manager,
                channel_manager,
                channel_monitor,
            };
            res
        }
    }

    pub async fn connect(client: Arc<LightingClient>) {
        let node_id = PublicKey::from_slice(
            &hex::decode("036b96e4713c5f84dcb8030592e1bd42a2d9a43d91fa2e535b9bfd05f2c5def9b9")
                .unwrap(),
        )
        .unwrap();
        let node_addr: SocketAddr = "38.87.54.163:9745".parse().unwrap();
        // let node_id = PublicKey::from_slice(
        //     &hex::decode("03933884aaf1d6b108397e5efe5c86bcf2d8ca8d2f700eda99db9214fc2712b134").unwrap(),
        // )
        // .unwrap();
        // let node_addr: SocketAddr = "34.250.234.192:9735".parse().unwrap();
        println!("{:?}", node_id);
        println!("{:?}", node_addr);
        let logger = Arc::new(log::ConsoleLogger::new());
        Arc::clone(&logger).log(&Record::new(
            Level::Info,
            format_args!("{}", "hello"),
            "module",
            "file",
            22,
        ));

        // let bitcoin_client = Arc::new(chain::FakeBitcoinClient::new(logger.clone()));
        // let tx = bitcoin::blockdata::transaction::Transaction {
        //     version: 2,
        //     lock_time: 0,
        //     input: vec![],
        //     output: vec![],
        // };
        // bitcoin_client.broadcast_transaction(&tx);

        // let channel_monitor = Arc::new(ChannelMonitor::new(
        //     bitcoin_client.clone(),
        //     bitcoin_client.clone(),
        //     logger.clone(),
        //     bitcoin_client.clone(),
        // ));

        // let secp_context = bitcoin::secp256k1::Secp256k1::new();
        // let funding_key = bitcoin::secp256k1::SecretKey::from_slice(&[0x01; 32]).unwrap();
        // let revocation_base_key = bitcoin::secp256k1::SecretKey::from_slice(&[0x02; 32]).unwrap();
        // let payment_key = bitcoin::secp256k1::SecretKey::from_slice(&[0x03; 32]).unwrap();
        // let delayed_payment_key = bitcoin::secp256k1::SecretKey::from_slice(&[0x04; 32]).unwrap();
        // let htlc_key = bitcoin::secp256k1::SecretKey::from_slice(&[0x5; 32]).unwrap();
        // let keys_manager = Arc::new(InMemoryChannelKeys::new(
        //     &secp_context,
        //     funding_key,
        //     revocation_base_key,
        //     payment_key,
        //     delayed_payment_key,
        //     htlc_key,
        //     [0x00; 32],
        //     0_u64,
        //     (0_u64, 0_u64),
        // ));

        // let ts = std::time::SystemTime::now()
        //     .duration_since(std::time::UNIX_EPOCH)
        //     .unwrap();
        // let seed = rand::random::<[u8; 32]>();
        // let keys_manager = Arc::new(KeysManager::new(
        //     &seed,
        //     Network::Testnet,
        //     ts.as_secs(),
        //     ts.subsec_nanos(),
        // ));

        // let user_config = lightning::util::config::UserConfig::default();
        // let channel_manager: ChannelManager =
        //     Arc::new(lightning::ln::channelmanager::ChannelManager::new(
        //         Network::Testnet,
        //         bitcoin_client.clone(),
        //         channel_monitor.clone(),
        //         bitcoin_client.clone(),
        //         logger.clone(),
        //         keys_manager,
        //         user_config,
        //         1000000,
        //     ));

        // // next construct the network graph manager. This imple
        // let net_graph_manager: NetGraphManager =
        //     Arc::new(lightning::routing::network_graph::NetGraphMsgHandler::new(
        //         bitcoin_client.clone(),
        //         logger.clone(),
        //     ));

        // // now that we have implemented the two types for message
        // // handling we can construct a MessageHandler which contains
        // // references to the different message handlers and it is
        // // used by the peer
        // let message_handler = lightning::ln::peer_handler::MessageHandler {
        //     chan_handler: channel_manager.clone(),
        //     route_handler: net_graph_manager.clone(),
        // };
        // let node_key = bitcoin::secp256k1::SecretKey::from_slice(
        //     &hex::decode("d84985781fee4676a616f81399d28cced95a691a983c582b6285108e02830673")
        //         .unwrap(),
        // )
        // .unwrap();
        // let peer_manager: PeerManager = Arc::new(lightning::ln::peer_handler::PeerManager::new(
        //     message_handler,
        //     node_key,
        //     &rand::random::<[u8; 32]>(),
        //     logger.clone(),
        // ));

        connect_to_node(
            client.peer_manager.clone(),
            client.channel_monitor.clone(),
            client.channel_manager.clone(),
            node_id,
            node_addr,
        )
        .await;
    }

    // Define concrete types for our high-level objects:
    type TxBroadcaster = dyn lightning::chain::chaininterface::BroadcasterInterface;
    type FeeEstimator = dyn lightning::chain::chaininterface::FeeEstimator;
    type ChainWatchInterface = dyn lightning::chain::chaininterface::ChainWatchInterface;
    type Logger = dyn lightning::util::logger::Logger;
    type ChannelMonitor = lightning::ln::channelmonitor::SimpleManyChannelMonitor<
        lightning::chain::transaction::OutPoint,
        lightning::chain::keysinterface::InMemoryChannelKeys,
        Arc<TxBroadcaster>,
        Arc<FeeEstimator>,
        Arc<Logger>,
        Arc<ChainWatchInterface>,
    >;
    type ChannelManager = lightning::ln::channelmanager::SimpleArcChannelManager<
        ChannelMonitor,
        TxBroadcaster,
        FeeEstimator,
        Logger,
    >;
    type NetGraphManager = Arc<
        lightning::routing::network_graph::NetGraphMsgHandler<
            Arc<ChainWatchInterface>,
            Arc<Logger>,
        >,
    >;
    type PeerManager = lightning::ln::peer_handler::SimpleArcPeerManager<
        lightning_net_tokio::SocketDescriptor,
        ChannelMonitor,
        TxBroadcaster,
        FeeEstimator,
        ChainWatchInterface,
        Logger,
    >;

    // Connect to node with pubkey their_node_id at addr:
    async fn connect_to_node(
        peer_manager: PeerManager,
        channel_monitor: Arc<ChannelMonitor>,
        channel_manager: ChannelManager,
        their_node_id: PublicKey,
        addr: SocketAddr,
    ) {
        let (sender, mut receiver) = mpsc::channel(2);
        lightning_net_tokio::connect_outbound(peer_manager, sender, their_node_id, addr).await;
        loop {
            receiver.recv().await;
            for _event in channel_manager.get_and_clear_pending_events().drain(..) {
                // Handle the event
            }
            for _event in channel_monitor.get_and_clear_pending_events().drain(..) {
                // Handle the event
            }
        }
    }

    // Begin reading from a newly accepted socket and talk to the peer:
    // async fn accept_socket(
    //     peer_manager: PeerManager,
    //     channel_monitor: Arc<ChannelMonitor>,
    //     channel_manager: ChannelManager,
    //     socket: TcpStream,
    // ) {
    //     let (sender, mut receiver) = mpsc::channel(2);
    //     lightning_net_tokio::setup_inbound(peer_manager, sender, socket);
    //     loop {
    //         receiver.recv().await;
    //         for _event in channel_manager.get_and_clear_pending_events().drain(..) {
    //             // Handle the event
    //         }
    //         for _event in channel_monitor.get_and_clear_pending_events().drain(..) {
    //             // Handle the event
    //         }
    //     }
    // }
}
