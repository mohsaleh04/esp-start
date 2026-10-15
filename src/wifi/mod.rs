pub use config::set_station_config;
pub use connection::connect;
pub use network::WifiNetwork;
pub use scanner::scan;

mod config;
mod connection;
mod network;
mod scanner;

use esp_hal::peripherals::WIFI;
use esp_radio::wifi::{ControllerConfig, Interface, WifiController, WifiError};

pub fn setup(
    wifi_periph: WIFI<'static>,
) -> Result<(WifiController<'static>, Interface), WifiError> {
    let controller = WifiController::new(wifi_periph, ControllerConfig::default())?;
    let station = Interface::station();
    Ok((controller, station))
}
