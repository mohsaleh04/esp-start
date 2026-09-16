use alloc::vec::Vec;
use esp_radio::wifi::ap::AccessPointInfo;
use esp_radio::wifi::scan::ScanConfig;
use esp_radio::wifi::WifiController;

pub async fn scan(controller: &mut WifiController<'static>) -> Vec<AccessPointInfo> {
    let config = ScanConfig::default().with_max(20);

    controller
        .scan_async(&config)
        .await
        .expect("WiFi scan failed")
}
