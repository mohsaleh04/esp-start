use esp_hal::peripherals::{GPIO1, GPIO3, UART0};
use esp_hal::uart::{Config, Uart};
use esp_hal::Blocking;

pub fn setup(
    uart: UART0<'static>,
    tx: GPIO1<'static>,
    rx: GPIO3<'static>,
) -> Uart<'static, Blocking> {
    Uart::new(uart, Config::default())
        .expect("Failed to setup UART")
        .with_tx(tx)
        .with_rx(rx)
}
