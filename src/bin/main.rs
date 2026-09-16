#![no_std]
#![no_main]

use core::fmt::Write;
use core::hint::spin_loop;
use core::net::Ipv4Addr;
use core::ops::ControlFlow;
use core::panic::PanicInfo;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::RefCellDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::pcnt::Pcnt;
use esp_hal::pcnt::channel::EdgeMode;
use esp_hal::timer::timg::TimerGroup;
use esp_start::com::uart;
use esp_start::io::{self, ButtonId, InputEvent, PinConfig, ScreenOutPins, SdOutPins};
use esp_start::leds::{LedController, UPDATE_INTERVAL_MS};
use esp_start::net::{HttpClient, WifiNetwork};
use esp_start::screen::{DEFAULT_CONTRAST, ScreenController, ScreenDriver};
use esp_start::sd::{SdStorage, SdStorageError};
use esp_start::{pcnt, runtime, spi_bus, timer};

const WIFI_SSID: &str = "HomeADSL";
const WIFI_PASSWORD: &str = "Home#1405";
const SERVER_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 101);
const SERVER_PORT: u16 = 80;
const HTTP_REQUEST: &[u8] = b"GET /gpio/1 HTTP/1.1\r\n\
    Host: 192.168.1.101\r\n\
    Connection: close\r\n\
    \r\n";

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        spin_loop();
    }
}

// Required application metadata for the ESP bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    runtime::allocate_heap();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut uart = uart::setup(peripherals.UART0, peripherals.GPIO1, peripherals.GPIO3);

    // --- Input and scheduling ---
    io::setup(peripherals.IO_MUX);
    io::setup_backlight_button(peripherals.GPIO32);
    io::setup_led_mode_button(peripherals.GPIO27);
    uart.write_str("[INPUT] Backlight button ready on GPIO32\r\n")
        .unwrap();
    uart.write_str("[INPUT] LED mode button ready on GPIO27\r\n")
        .unwrap();

    let timer_group = TimerGroup::new(peripherals.TIMG0);
    timer::setup(timer_group.timer0, UPDATE_INTERVAL_MS);
    runtime::setup_scheduler(peripherals.SW_INTERRUPT, timer_group.timer1);

    // GPIO25 blinks. GPIO26 and GPIO14 are the complementary PWM fade pair.
    let mut leds = LedController::new(
        peripherals.LEDC,
        peripherals.GPIO25,
        peripherals.GPIO26,
        peripherals.GPIO14,
    );

    // --- Shared SPI bus ---
    let spi_bus = spi_bus::setup(
        peripherals.SPI2,
        peripherals.GPIO18, // SCK
        peripherals.GPIO23, // MOSI
        peripherals.GPIO19, // MISO (SD)
    );

    // --- LCD ---
    uart.write_str("[LCD] Initializing ... ").unwrap();
    let screen_out_pins = ScreenOutPins::new(
        peripherals.GPIO22, // backlight
        peripherals.GPIO21, // reset
        peripherals.GPIO17, // data/command
        peripherals.GPIO5,  // chip select
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
                spin_loop();
            }
        }
    };
    screen.clear();
    if let Err(error) = screen.flush() {
        uart.write_str("FAILED\r\n").unwrap();
        writeln!(uart, "--> Error: {error:#?}").unwrap();
        loop {
            spin_loop();
        }
    }
    screen.toggle_backlight();
    uart.write_str("SUCCESS\r\n").unwrap();

    // --- SD card ---
    uart.write_str("[SD] Mounting ... ").unwrap();
    let sd_out_pins = SdOutPins::new(peripherals.GPIO16);
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

    // --- Wi-Fi and HTTP ---
    let wifi_network = WifiNetwork::connect(
        peripherals.WIFI,
        spawner,
        WIFI_SSID,
        Some(WIFI_PASSWORD),
        &mut uart,
    )
    .await;
    let mut http_client = match wifi_network.as_ref() {
        Some(network) => Some(
            HttpClient::connect_and_send(
                network.stack(),
                SERVER_IP,
                SERVER_PORT,
                HTTP_REQUEST,
                &mut uart,
            )
            .await,
        ),
        None => None,
    };

    // --- Pulse counter on GPIO33 ---
    let pulse_counter = Pcnt::new(peripherals.PCNT);
    let pulse_unit = pulse_counter.unit0;
    pcnt::setup(
        &pulse_unit.channel0,
        peripherals.GPIO33,
        PinConfig::PullUp.as_input(),
        EdgeMode::Increment,
        EdgeMode::Hold,
    );
    pulse_unit.clear();
    pulse_unit.resume();
    let mut last_pulse_count = pulse_unit.value();

    loop {
        while let Some(event) = io::next_input_event() {
            match event {
                InputEvent::ButtonPressed(ButtonId::Backlight) => {
                    screen.toggle_backlight();
                    uart.write_str("[INPUT] Backlight toggled\r\n").unwrap();
                }
                InputEvent::ButtonPressed(ButtonId::LedMode) => {
                    let mode = leds.cycle_mode();
                    writeln!(uart, "[INPUT] LED mode: {mode:?}").unwrap();
                }
                InputEvent::ButtonReleased(button) => {
                    writeln!(uart, "[INPUT] {button:?} button released").unwrap();
                }
            }
        }

        leds.update();

        let pulse_count = pulse_unit.value();
        if pulse_count != last_pulse_count {
            writeln!(uart, "[PCNT] Current value: {pulse_count}").unwrap();
            last_pulse_count = pulse_count;
        }

        if let Some(client) = http_client.as_mut() {
            client.poll(&mut uart).await;
        }

        // Yield so the Embassy network runner can execute even with no TCP traffic.
        Timer::after(Duration::from_millis(1)).await;
    }
}
