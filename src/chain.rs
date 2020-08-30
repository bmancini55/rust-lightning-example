use bitcoin::Transaction;
use lightning::chain::chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator};
use lightning::util::logger::{Level, Logger, Record};
use std::ops::Deref;

pub struct FakeBitcoinClient<L: Deref>
where
    L::Target: Logger,
{
    pub logger: L,
}

unsafe impl<L: Deref> Sync for FakeBitcoinClient<L> where L::Target: Logger {}
unsafe impl<L: Deref> Send for FakeBitcoinClient<L> where L::Target: Logger {}

impl<L: Deref> FakeBitcoinClient<L>
where
    L::Target: Logger,
{
    pub fn new(logger: L) -> FakeBitcoinClient<L> {
        FakeBitcoinClient { logger }
    }
}

impl<L: Deref> BroadcasterInterface for FakeBitcoinClient<L>
where
    L::Target: Logger,
{
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

impl<L: Deref> FeeEstimator for FakeBitcoinClient<L>
where
    L::Target: Logger,
{
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
