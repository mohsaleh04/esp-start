use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config, Password, Ssid, WifiController, WifiError,
};

pub fn set_station_config(
    controller: &mut WifiController,
    ssid: &str,
    password: Option<&str>,
) -> Result<(), WifiError> {
    let ssid = Ssid::try_from(ssid)?;
    let authentication = match password {
        Some(password) => AuthenticationMethodConfig::Wpa2Personal(Password::try_from(password)?),
        None => AuthenticationMethodConfig::Open,
    };
    let config = Config::Station(
        StationConfig::default()
            .with_ssid(ssid)
            .with_authentication(authentication),
    );

    controller.set_config(&config)
}
