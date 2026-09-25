use crate::bluetooth::advertisement::AdvertisementData;
use bt_hci::param::BdAddr;

#[derive(Debug, Clone)]
pub struct BleDevice {
    pub address: BdAddr,
    pub rssi: i8,
    pub adv: AdvertisementData,
}
