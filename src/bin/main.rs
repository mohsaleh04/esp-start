#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::panic::PanicInfo;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_start::com::uart;

// #############

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

    let mut uart = uart::setup(_peripherals.UART0, _peripherals.GPIO1, _peripherals.GPIO3);

    loop {}
}
