#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use embedded_hal_bus::spi::RefCellDevice;
use embedded_sdmmc::SdCard;
use esp_hal::delay::Delay;
use esp_hal::{clock::CpuClock, main};
use esp_start::com::uart;
use esp_start::io::{ScreenOutPins, SdOutPins};
use esp_start::screen::{DEFAULT_CONTRAST, ScreenController, ScreenDriver};
use esp_start::spi_bus;
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
    uart.write_str("\r\n").unwrap();

    let spi_bus = spi_bus::setup(
        peripherals.SPI2,
        peripherals.GPIO18, // SCK
        peripherals.GPIO23, // MOSI
        peripherals.GPIO19, // MISO (for SD)
    );

    // --- LCD ---
    uart.write_str("[LCD] Initializing ... ").unwrap();
    let screen_out_pins = ScreenOutPins::new(
        peripherals.GPIO22, // backlight
        peripherals.GPIO21, // rst
        peripherals.GPIO17, // dc
        peripherals.GPIO5,  // cs
    );

    let screen_res = ScreenController::init(
        DEFAULT_CONTRAST,
        ScreenDriver::new(
            RefCellDevice::new(&spi_bus, screen_out_pins.cs, Delay::new()).unwrap(),
            screen_out_pins.screen,
        ),
    );
    if let Err(e) = screen_res {
        uart.write_str("FAILED\r\n").unwrap();
        write!(uart, "--> Error: ${:#?}", e).unwrap();
        loop {}
    }
    let mut screen = screen_res.unwrap();
    if screen.clear().is_err() {
        uart.write_str("FAILED\r\n").unwrap();
        write!(uart, "--> Error: Failed to clear screen").unwrap();
        loop {}
    }
    screen.toggle_backlight();
    uart.write_str("SUCCESS\r\n").unwrap();

    // --- SD Card ---
    uart.write_str("[SD] Initializing ... ").unwrap();
    let sd_out_pins = SdOutPins::new(
        peripherals.GPIO16, // cs
    );
    let sd_spi = RefCellDevice::new(&spi_bus, sd_out_pins.cs, Delay::new()).unwrap();
    let sd = SdCard::new(sd_spi, Delay::new());

    let sd_size: u64;
    match sd.num_bytes() {
        Ok(size) => {
            sd_size = size;
            uart.write_str("SUCCESS\r\n").unwrap();
            writeln!(uart, "[SD] Size: {} MB", size / 1048576).unwrap();
        }

        Err(err) => {
            uart.write_str("FAILED\r\n").unwrap();
            writeln!(uart, "--> Error: {:?}", err).unwrap();
            loop {}
        }
    }

    screen.draw_fmt(
        (0, 0),
        format_args!("Setup Complete\nSD Size: {} MB", sd_size / 1048576),
        true,
        false,
    );
    delay(10);

    loop {}
}
