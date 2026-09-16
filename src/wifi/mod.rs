pub mod config;
pub mod scanner;
pub mod connection;

use esp_hal::peripherals::WIFI;

use esp_radio::wifi::{ControllerConfig, Interfaces, WifiController};

pub fn setup(wifi: WIFI<'static>) -> (WifiController<'static>, Interfaces<'static>) {
    esp_radio::wifi::new(wifi, ControllerConfig::default()).expect("Couldn't setup WiFi")
}
