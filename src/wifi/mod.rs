pub mod config;
pub mod connection;
pub mod scanner;

use esp_hal::peripherals::WIFI;
use esp_radio::wifi::{ControllerConfig, Interface, WifiController, WifiError};

pub fn setup(wifi: WIFI<'static>) -> Result<(WifiController<'static>, Interface), WifiError> {
    let controller = WifiController::new(wifi, ControllerConfig::default())?;
    let station = Interface::station();
    Ok((controller, station))
}
