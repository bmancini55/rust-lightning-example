use crate::chain;
use crate::log;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1::key::{PublicKey, SecretKey};
use lightning::chain::keysinterface::KeysManager;
use lightning::util::events::EventsProvider;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

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
    lightning::routing::network_graph::NetGraphMsgHandler<Arc<ChainWatchInterface>, Arc<Logger>>,
>;
type PeerManager = lightning::ln::peer_handler::SimpleArcPeerManager<
    lightning_net_tokio::SocketDescriptor,
    ChannelMonitor,
    TxBroadcaster,
    FeeEstimator,
    ChainWatchInterface,
    Logger,
>;

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
        let peer_manager: PeerManager = Arc::new(lightning::ln::peer_handler::PeerManager::new(
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

impl LightingClient {
    // Connect to node with pubkey their_node_id at addr:
    pub async fn connect_to_node(&self, their_node_id: PublicKey, addr: SocketAddr) {
        let (sender, mut receiver) = mpsc::channel(2);

        lightning_net_tokio::connect_outbound(
            self.peer_manager.clone(),
            sender,
            their_node_id,
            addr,
        )
        .await;

        loop {
            receiver.recv().await;
            for _event in self
                .channel_manager
                .get_and_clear_pending_events()
                .drain(..)
            {
                // Handle the event
            }
            for _event in self
                .channel_monitor
                .get_and_clear_pending_events()
                .drain(..)
            {
                // Handle the event
            }
        }
    }
}
