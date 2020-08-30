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
