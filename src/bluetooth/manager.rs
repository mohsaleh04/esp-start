use embassy_executor::Spawner;
use crate::bluetooth::{DeviceRegistry, advertisement, scanner};
use esp_hal::peripherals::BT;
use crate::bluetooth;

pub struct BluetoothManager {
    devices: DeviceRegistry,
}

impl BluetoothManager {
    pub fn init(bt: BT<'static>, spawner: &Spawner) -> Self {
        bluetooth::run_bt_scan_task(spawner, bt);
        Self {
            devices: DeviceRegistry::new(),
        }
    }

    pub fn devices(&self) -> &DeviceRegistry {
        &self.devices
    }

    pub fn poll(&mut self) -> bool {
        let mut changed = false;

        while let Some(result) = scanner::next_scan_result() {
            let parsed = advertisement::parse_advertisement_data(&result);

            if self.devices.update(result.address, result.rssi, parsed) {
                changed = true;
            }
        }

        changed
    }
}
