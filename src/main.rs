#[tokio::main]
async fn main() {
    client::run().await;
}

mod client {
    use super::chain;
    use super::log;
    use bitcoin::network::constants::Network;
    use bitcoin::secp256k1::key::PublicKey;
    use hex;
    use lightning::chain::chaininterface::BroadcasterInterface as BroadcasterInterfaceTrait;
    use lightning::chain::keysinterface::KeysManager;
    use lightning::util::events::EventsProvider;
    use lightning::util::logger::Logger as LoggerTrait;
    use lightning::util::logger::{Level, Record};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    pub async fn run() {
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
        let bitcoin_client = Arc::new(chain::FakeBitcoinClient::new(logger.clone()));
        let tx = bitcoin::blockdata::transaction::Transaction {
            version: 2,
            lock_time: 0,
            input: vec![],
            output: vec![],
        };
        bitcoin_client.broadcast_transaction(&tx);
        let channel_monitor = Arc::new(ChannelMonitor::new(
            bitcoin_client.clone(),
            bitcoin_client.clone(),
            logger.clone(),
            bitcoin_client.clone(),
        ));
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
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let seed = rand::random::<[u8; 32]>();
        let keys_manager = Arc::new(KeysManager::new(
            &seed,
            Network::Testnet,
            ts.as_secs(),
            ts.subsec_nanos(),
        ));
        let user_config = lightning::util::config::UserConfig::default();
        let channel_manager: ChannelManager =
            Arc::new(lightning::ln::channelmanager::ChannelManager::new(
                Network::Testnet,
                bitcoin_client.clone(),
                channel_monitor.clone(),
                bitcoin_client.clone(),
                logger.clone(),
                keys_manager,
                user_config,
                1000000,
            ));
        let net_graph_manager: NetGraphManager =
            Arc::new(lightning::routing::network_graph::NetGraphMsgHandler::new(
                bitcoin_client.clone(),
                logger.clone(),
            ));
        let message_handler = lightning::ln::peer_handler::MessageHandler {
            chan_handler: channel_manager.clone(),
            route_handler: net_graph_manager.clone(),
        };
        let node_key = bitcoin::secp256k1::SecretKey::from_slice(
            &hex::decode("d84985781fee4676a616f81399d28cced95a691a983c582b6285108e02830673")
                .unwrap(),
        )
        .unwrap();
        let peer_manager: PeerManager = Arc::new(lightning::ln::peer_handler::PeerManager::new(
            message_handler,
            node_key,
            &rand::random::<[u8; 32]>(),
            logger.clone(),
        ));
        connect_to_node(
            peer_manager,
            channel_monitor,
            channel_manager,
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

mod log {
    use lightning::util::logger::{Logger, Record};

    pub struct ConsoleLogger {}

    impl ConsoleLogger {
        pub fn new() -> ConsoleLogger {
            ConsoleLogger {}
        }
    }

    unsafe impl Send for ConsoleLogger {}
    unsafe impl Sync for ConsoleLogger {}

    impl Logger for ConsoleLogger {
        fn log(&self, record: &Record) {
            println!("{} {:?}", record.level, record.args);
        }
    }
}

mod chain {
    use bitcoin::{Block, BlockHash, Script, Transaction, Txid};
    use lightning::chain::chaininterface::{
        BroadcasterInterface, ChainError, ChainWatchInterface, ConfirmationTarget, FeeEstimator,
    };
    use lightning::util::logger::{Level, Logger, Record};
    use std::sync::Arc;

    pub struct FakeBitcoinClient {
        pub logger: Arc<dyn Logger>,
    }

    unsafe impl Sync for FakeBitcoinClient {}
    unsafe impl Send for FakeBitcoinClient {}

    impl FakeBitcoinClient {
        pub fn new(logger: Arc<dyn Logger>) -> FakeBitcoinClient {
            FakeBitcoinClient { logger }
        }
    }

    impl BroadcasterInterface for FakeBitcoinClient {
        /// Sends a transaction out to (hopefully) be mined.
        fn broadcast_transaction(&self, tx: &Transaction) {
            self.logger.log(&Record::new(
                Level::Info,
                format_args!("{:?}", tx),
                "chain",
                "FakeBitcoinClient",
                0,
            ));
        }
    }

    impl FeeEstimator for FakeBitcoinClient {
        /// Gets estimated satoshis of fee required per 1000 Weight-Units.
        ///
        /// Must be no smaller than 253 (ie 1 satoshi-per-byte rounded up to ensure later round-downs
        /// don't put us below 1 satoshi-per-byte).
        ///
        /// This translates to:
        ///  * satoshis-per-byte * 250
        ///  * ceil(satoshis-per-kbyte / 4)
        fn get_est_sat_per_1000_weight(&self, _confirmation_target: ConfirmationTarget) -> u32 {
            return 1000;
        }
    }

    impl ChainWatchInterface for FakeBitcoinClient {
        /// Provides a txid/random-scriptPubKey-in-the-tx which much be watched for.
        fn install_watch_tx(&self, _txid: &Txid, _script_pub_key: &Script) {
            // no-op
        }

        /// Provides an outpoint which must be watched for, providing any transactions which spend the
        /// given outpoint.
        fn install_watch_outpoint(&self, _outpoint: (Txid, u32), _out_script: &Script) {
            // no-op
        }

        /// Indicates that a listener needs to see all transactions.
        fn watch_all_txn(&self) {
            // no-op
        }

        /// Gets the script and value in satoshis for a given unspent transaction output given a
        /// short_channel_id (aka unspent_tx_output_identier). For BTC/tBTC channels the top three
        /// bytes are the block height, the next 3 the transaction index within the block, and the
        /// final two the output within the transaction.
        fn get_chain_utxo(
            &self,
            _genesis_hash: BlockHash,
            _unspent_tx_output_identifier: u64,
        ) -> Result<(Script, u64), ChainError> {
            Ok((Script::new(), 0))
        }

        /// Gets the list of transaction indices within a given block that the ChainWatchInterface is
        /// watching for.
        fn filter_block(&self, _block: &Block) -> Vec<usize> {
            // no-op
            vec![]
        }

        /// Returns a usize that changes when the ChainWatchInterface's watched data is modified.
        /// Users of `filter_block` should pre-save a copy of `reentered`'s return value and use it to
        /// determine whether they need to re-filter a given block.
        fn reentered(&self) -> usize {
            0
        }
    }
}
