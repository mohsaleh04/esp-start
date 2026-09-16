#![no_std]
#![no_main]

use core::fmt::Write;
use core::ops::ControlFlow;
use core::panic::PanicInfo;
use embedded_hal_bus::spi::RefCellDevice;
use esp_hal::delay::Delay;
use esp_hal::{clock::CpuClock, main};
use esp_start::com::uart;
use esp_start::io::{self, ButtonId, InputEvent, ScreenOutPins, SdOutPins};
use esp_start::screen::{DEFAULT_CONTRAST, ScreenController, ScreenDriver};
use esp_start::sd::{SdStorage, SdStorageError};
use esp_start::spi_bus;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

// Don't remove this
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut uart = uart::setup(peripherals.UART0, peripherals.GPIO1, peripherals.GPIO3);

    io::setup(peripherals.IO_MUX);
    io::setup_primary_button(peripherals.GPIO32);
    uart.write_str("[INPUT] Primary button ready on GPIO32\r\n").unwrap();
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
    let mut screen = match screen_res {
        Ok(screen) => screen,
        Err(error) => {
            uart.write_str("FAILED\r\n").unwrap();
            writeln!(uart, "--> Error: {error:#?}").unwrap();
            loop {
                core::hint::spin_loop();
            }
        }
    };
    screen.clear();
    if let Err(error) = screen.flush() {
        uart.write_str("FAILED\r\n").unwrap();
        writeln!(uart, "--> Error: {error:#?}").unwrap();
        loop {
            core::hint::spin_loop();
        }
    }
    screen.toggle_backlight();
    uart.write_str("SUCCESS\r\n").unwrap();

    // --- SD Card ---
    uart.write_str("[SD] Mounting ... ").unwrap();
    let sd_out_pins = SdOutPins::new(peripherals.GPIO16); // cs
    let sd_spi = RefCellDevice::new(&spi_bus, sd_out_pins.cs, Delay::new()).unwrap();
    let storage = SdStorage::new(sd_spi, Delay::new());

    match storage.mount() {
        Ok(sd) => {
            uart.write_str("SUCCESS\r\n").unwrap();
            let size_mb = sd.size_bytes() / 1_048_576;
            writeln!(uart, "[SD] Size: {size_mb} MB").unwrap();

            screen.draw_fmt(
                (0, 0),
                format_args!("Setup Complete\nSD Size: {size_mb} MB"),
                true,
                false,
            );

            if let Err(error) = sd.for_each_root_entry(|entry| {
                writeln!(
                    uart,
                    "{} {:?} {} bytes",
                    entry.name, entry.attributes, entry.size
                )
                .unwrap();
                ControlFlow::Continue(())
            }) {
                match error {
                    SdStorageError::CardNotFound => writeln!(uart, "[SD] Card removed").unwrap(),
                    error => writeln!(uart, "[SD] Root directory read failed: {error:?}").unwrap(),
                }
            }
        }
        Err(SdStorageError::CardNotFound) => {
            uart.write_str("NOT FOUND\r\n").unwrap();
            screen.draw_text((0, 0), "Setup Complete\nSD not inserted", true, false);
        }
        Err(error) => {
            uart.write_str("FAILED\r\n").unwrap();
            writeln!(uart, "--> Error: {error:?}").unwrap();
            screen.draw_text((0, 0), "Setup Complete\nSD mount failed", true, false);
        }
    }

    if let Err(error) = screen.flush() {
        writeln!(uart, "[LCD] Flush failed: {error:#?}").unwrap();
    }

    loop {
        while let Some(event) = io::next_input_event() {
            match event {
                InputEvent::ButtonPressed(ButtonId::Primary) => {
                    uart.write_str("[INPUT] Primary button pressed\r\n")
                        .unwrap();
                    screen.toggle_backlight();
                }
                InputEvent::ButtonReleased(ButtonId::Primary) => {
                    uart.write_str("[INPUT] Primary button released\r\n")
                        .unwrap();
                }
            }
        }

        core::hint::spin_loop();
    }
}
