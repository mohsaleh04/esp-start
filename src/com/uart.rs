use esp_hal::Blocking;
use esp_hal::peripherals::{GPIO1, GPIO3, UART0};
use esp_hal::uart::{Config, Uart};

pub fn setup(
    uart: UART0<'static>,
    tx: GPIO1<'static>,
    rx: GPIO3<'static>,
) -> Uart<'static, Blocking> {
    Uart::new(uart, Config::default())
        .unwrap()
        .with_tx(tx)
        .with_rx(rx)
}
