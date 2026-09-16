use core::fmt::Write;
use esp_hal::Blocking;
use esp_hal::uart::Uart;
use esp_radio::wifi::{ConnectionError, WifiController, WifiError};

pub async fn connect(
    controller: &mut WifiController<'static>,
    uart: &mut Uart<'static, Blocking>,
) -> bool {
    match controller.connect_async().await {
        Ok(info) => {
            uart.write_str("Wifi Connected!\r\n").unwrap();
            write!(uart, "Wifi Connection: {info:?}\r\n").unwrap();
            true
        }
        Err(ConnectionError::Failed(disconnected)) => {
            write!(uart, "Connection failed: {:?}\r\n", disconnected.reason).unwrap();
            false
        }
        Err(ConnectionError::WifiError(error)) => {
            match error {
                WifiError::OutOfMemory => {
                    uart.write_str("Connection failed: Out of Memory!\r\n")
                        .unwrap();
                }
                WifiError::InvalidSsid => {
                    uart.write_str("Connection failed: Invalid SSID\r\n")
                        .unwrap();
                }
                WifiError::InvalidPassword => {
                    uart.write_str("Connection failed: Invalid Password\r\n")
                        .unwrap();
                }
                error => {
                    write!(uart, "Connection failed: {error:?}\r\n").unwrap();
                }
            }
            false
        }
        Err(error) => {
            write!(uart, "Connection failed: {error:?}\r\n").unwrap();
            false
        }
    }
}
