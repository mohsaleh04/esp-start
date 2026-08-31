use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{Config, Ssid, WifiController};

pub fn set_station_config(
    controller: &mut WifiController,
    ssid: impl Into<Ssid>,
    password: Option<&str>,
) {
    let station_config = match password {
        Some(pass) => StationConfig::default()
            .with_ssid(ssid)
            .with_password(pass.into()),
        _ => StationConfig::default().with_ssid(ssid),
    };
    let wifi_config = Config::Station(station_config);
    controller
        .set_config(&wifi_config)
        .expect("Couldn't register config for connection\r\n");
}
