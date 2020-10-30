use lightning::chain::channelmonitor::ChannelMonitorUpdate;
use lightning::chain::channelmonitor::ChannelMonitorUpdateErr;
use lightning::chain::channelmonitor::ChannelMonitor;
use lightning::chain::keysinterface::InMemoryChannelKeys;
use lightning::chain::channelmonitor::Persist;
use lightning::chain::transaction::OutPoint;

pub struct ChannelPersistor {

}

impl Persist<InMemoryChannelKeys> for ChannelPersistor where
{
    fn persist_new_channel(&self, _id: OutPoint, _data: &ChannelMonitor<InMemoryChannelKeys>) -> Result<(), ChannelMonitorUpdateErr> {
        Ok(())
    }

    fn update_persisted_channel(&self, _id: OutPoint, _update: &ChannelMonitorUpdate, _data: &ChannelMonitor<InMemoryChannelKeys>) -> Result<(), ChannelMonitorUpdateErr> {
        Ok(())
    }
}