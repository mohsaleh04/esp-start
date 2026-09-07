#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp_hal::{clock::CpuClock, main};
use esp_start::com::uart;
use esp_start::io::ScreenOutPins;
use esp_start::screen::{ScreenController, ScreenDriver};
use esp_start::utils::delay;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

// Don't remove this
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut uart = uart::setup(peripherals.UART0, peripherals.GPIO1, peripherals.GPIO3);
    uart.write_str("[LCD] Initializing ... ").unwrap();

    let screen_out_pins = ScreenOutPins::new(
        peripherals.GPIO22, // backlight
        peripherals.GPIO21, // rst
        peripherals.GPIO19, // dc
        peripherals.GPIO5,  // cs
    );
    let mut screen = ScreenController::init(
        0x36,
        ScreenDriver::new(
            peripherals.SPI2,
            screen_out_pins.backlight,
            screen_out_pins.dc,
            screen_out_pins.cs,
            screen_out_pins.rst,
            peripherals.GPIO18, // SCK
            peripherals.GPIO23, // MOSI
        ),
    );

    screen.toggle_backlight();
    uart.write_str("SUCCESS\r\n").unwrap();

    screen.clear();
    screen.draw_rect((0, 0), 20, 20, true);
    screen.draw_circle((24, 24), 15, true);
    screen.draw_round_rect((32, 32), 30, 15, 3, true);
    delay(10);

    loop {}
}
