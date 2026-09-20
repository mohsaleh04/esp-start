use crate::bluetooth::advertisement::AdvertisementData;
use crate::bluetooth::models::BleDevice;
use bt_hci::param::BdAddr;

#[derive(Debug, Clone)]
pub struct DeviceRegistry {
    devices: heapless::Vec<BleDevice, 16>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: heapless::Vec::new(),
        }
    }

    pub fn update(&mut self, address: BdAddr, rssi: i8, adv: AdvertisementData) -> bool {
        if let Some(device) = self.devices.iter_mut()
            .find(|device| device.address.raw() == address.raw()) {

            let result_rssi = device.rssi.wrapping_sub(rssi).abs() >= 12;
            device.rssi = rssi;

            let result_adv = device.adv.merge(adv);
            return result_rssi || result_adv;
        }

        self.devices
            .push(BleDevice { address, rssi, adv })
            .expect("Failed to update this device in registry!");
        true
    }
}
