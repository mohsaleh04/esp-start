#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::fmt::Write;
use core::panic::PanicInfo;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::uart::{Config, Uart};
use esp_start::delay;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

// Don't Remove This Code! Code for bootloader
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    let mut uart = Uart::new(_peripherals.UART0, Config::default())
        .unwrap()
        .with_tx(_peripherals.GPIO1)
        .with_rx(_peripherals.GPIO3);

    let mut led = Output::new(_peripherals.GPIO23, Level::Low, OutputConfig::default());


    uart.write_str("==== SALEH ESP32 ====\r\n\tHello from Rust on ESP32!\r\n")
        .unwrap();

    let mut mulp = 1;

    loop {

        led.set_high();
        uart.write_str("LED -> ON\r\n").unwrap();
        delay(10 * mulp);

        led.set_low();
        uart.write_str("LED -> OFF\r\n").unwrap();
        delay(10 * mulp);

        mulp += 1;
        write!(uart, "mulp = {}\r\n", mulp).unwrap();
    }
}
