#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp_hal::{clock::CpuClock, main};
use esp_start::com::uart;
use esp_start::io::ScreenOutPins;
use esp_start::screen::{Screen, ScreenController, SCREEN_HEIGHT, SCREEN_WIDTH};

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
    let screen_out_pins = ScreenOutPins::new(
        peripherals.GPIO22, // backlight
        peripherals.GPIO21, // rst
        peripherals.GPIO19, // dc
        peripherals.GPIO5,  // cs
    );

    let mut screen = Screen::new(ScreenController::new(
        peripherals.SPI2,
        screen_out_pins.backlight,
        screen_out_pins.dc,
        screen_out_pins.cs,
        screen_out_pins.rst,
        peripherals.GPIO18, // SCK
        peripherals.GPIO23, // MOSI
    ));

    uart.write_str("[LCD] Initializing ...\r\n").unwrap();
    screen.init(0x36);

    screen.clear();
    screen.toggle_backlight();

    let mut swap = false;
    loop {
        for y in 0..SCREEN_HEIGHT as u8 {
            for x in 0..SCREEN_WIDTH as u8 {
                screen.set_pixel(x, y, !swap);
                write!(uart, "[LCD] x:{} y:{} swap:{}\r\n", x, y, swap).unwrap();
            }
        }
        swap = !swap;
    }
}
