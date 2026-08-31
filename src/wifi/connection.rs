use core::fmt::Write;
use esp_hal::Blocking;
use esp_hal::uart::Uart;
use esp_radio::wifi::{WifiController, WifiError};

pub async fn connect(controller: &mut WifiController<'static>, uart: &mut Uart<'static, Blocking>) -> bool {
    let mut wifi_connection_success = false;
    match controller.connect_async().await {
        Ok(info) => {
            uart.write_str("Wifi Connected!\r\n").unwrap();
            write!(uart, "Wifi Connection: {:?}\r\n", info).unwrap();
            wifi_connection_success = true;
        }

        Err(error) => {
            match error {
                WifiError::Disconnected(disc) => {
                    write!(uart, "Connection failed: {:?}\r\n", disc.reason).unwrap();
                }
                WifiError::Failed => {
                    write!(uart, "Failed to connect to this WIFI network!\r\n").unwrap();
                }
                WifiError::OutOfMemory => {
                    write!(uart, "Connection failed: Out of Memory!\r\n").unwrap();
                }
                WifiError::InvalidSsid => {
                    write!(uart, "Connection failed: Invalid SSID\r\n").unwrap();
                }
                WifiError::InvalidPassword => {
                    write!(uart, "Connection failed: Invalid Password\r\n").unwrap();
                }
                err => {
                    write!(uart, "Connection failed: {:?}\r\n", err).unwrap();
                }
            }
        }
    }
    wifi_connection_success
}