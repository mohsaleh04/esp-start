use bt_hci::param::BdAddr;
use crate::bluetooth::advertisement::AdvertisementData;

#[derive(Debug, Clone)]
pub struct BleDevice {
    pub address: BdAddr,
    pub rssi: i8,
    pub adv: AdvertisementData,
}
