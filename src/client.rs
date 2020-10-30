use crate::channelpersistor::ChannelPersistor;
use crate::chain;

use bitcoin::network::constants::Network;
use bitcoin::secp256k1::key::{PublicKey, SecretKey};
use lightning::chain::keysinterface::InMemoryChannelKeys;
use lightning::chain::keysinterface::KeysManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::net::{TcpListener};



// Define concrete types for our high-level objects:
type Logger = dyn lightning::util::logger::Logger;
type TxBroadcaster = dyn lightning::chain::chaininterface::BroadcasterInterface;
type FeeEstimator = dyn lightning::chain::chaininterface::FeeEstimator;
type ChainFilter = dyn lightning::chain::Filter;
type ChainAccess = dyn lightning::chain::Access;

type ChainMonitor = lightning::chain::chainmonitor::ChainMonitor<
    InMemoryChannelKeys,
    Arc<ChainFilter>,
    Arc<TxBroadcaster>,
    Arc<FeeEstimator>,
    Arc<Logger>,
    Arc<ChannelPersistor>
>;

type ChannelManager = lightning::ln::channelmanager::SimpleArcChannelManager<
    ChainMonitor,
    TxBroadcaster,
    FeeEstimator,
    Logger,
>;

type NetGraphManager = lightning::routing::network_graph::NetGraphMsgHandler<
    Arc<ChainAccess>,
    Arc<Logger>
>;

type PeerManager = lightning::ln::peer_handler::SimpleArcPeerManager<
    lightning_net_tokio::SocketDescriptor,
    ChainMonitor,
    TxBroadcaster,
    FeeEstimator,
    ChainAccess,
    Logger,
>;

/// Super basic wrapper class that is going to implement the bare
/// minimum needed so we can look for p2p messaging traffic and
/// connect to a remote node.
pub struct LightingClient {
    peer_manager: PeerManager,
    logger: Arc<Logger>,
}

impl LightingClient {
    pub fn new(
        node_key: SecretKey,
        seed: &[u8; 32],
        user_config: lightning::util::config::UserConfig,
        network: Network,
        logger: Arc<Logger>,
    ) -> Self {

        // Constructs a fake bitcoin_client which implements FeeEstimator
        // and TransactionBroadcaster.
        let bitcoin_client = Arc::new(chain::FakeBitcoinClient::new(logger.clone()));

        // Constructs a fake channel persistor since we're only concerned
        // with gossip related stuff.
        let peristor = Arc::new(ChannelPersistor{});

        // Construct a ChainMonitor that does not use a chain::Filter,
        // also supply the FeeEstimator, TxBroadcaster, and a Logger
        let chain_monitor: Arc<ChainMonitor> = Arc::new(
            lightning::chain::chainmonitor::ChainMonitor::new(
                None,
                bitcoin_client.clone(),
                logger.clone(),
                bitcoin_client.clone(),
                peristor.clone(),
            )
        );

        // Next we construct a keys_manager from our supplied seed
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

        // Next we construct a channel manager for for the Testnet
        // network and supply FeeEstimator, TxBroadcaster, ChainMonitor,
        // Logger, and KeyManager. This type implements ChannelMessageHandler
        // and is used by the message_handler which gets used in the
        // PeerManager.
        let last_height: usize = 1000000;
        let channel_manager: ChannelManager = Arc::new(
            lightning::ln::channelmanager::ChannelManager::new(
                Network::Testnet,
                bitcoin_client.clone(),
                chain_monitor.clone(),
                bitcoin_client.clone(),
                logger.clone(),
                keys_manager.clone(),
                user_config,
                last_height,
            )
        );

        // Next construct the NetGraphMsgHandler which requires a
        // chain::Access. This type will be used as the
        // RoutingMessageHandler which gets attached to a MessageHandler,
        // which is itself used in the PeerManager. Cool.
        let net_graph_manager: Arc<NetGraphManager> =
            Arc::new(lightning::routing::network_graph::NetGraphMsgHandler::new(
                None,
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

        // Now that we have the MessageHandler constructed we can
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
            logger: logger.clone(),
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
            self.peer_manager.process_events();
        }
    }

    pub async fn listen(&self, addr_str: &str) -> Result<TcpListener, Box<dyn std::error::Error>>   {
        let addr: SocketAddr = addr_str.parse().unwrap();
        let mut listener = TcpListener::bind(addr).await?;
        log_info!(self.logger, "listening for sockets on {}", addr_str);
        loop {
            let (socket, _) = listener.accept().await?;
            let peer_manager = self.peer_manager.clone();
            tokio::spawn(async move {
                let (sender, mut receiver) = mpsc::channel(2);
                lightning_net_tokio::setup_inbound(peer_manager.clone(), sender, socket).await;

                loop {
                    receiver.recv().await;
                    peer_manager.process_events();
                }
            });
        }
    }
}
