#![no_std]
#![no_main]
extern crate alloc;

use alloc::string::ToString;
use core::fmt::Write;
use core::hint::spin_loop;
use core::net::Ipv4Addr;
use core::ops::ControlFlow;
use core::panic::PanicInfo;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::spi::SpiBus;
use embedded_hal_bus::spi::RefCellDevice;
use esp_hal::Blocking;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::pcnt::Pcnt;
use esp_hal::spi::master::Spi;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::Uart;
use esp_start::ai::{AiChatRequest, AiClient, AiError};
use esp_start::bluetooth::{BluetoothEvent, BluetoothManager};
use esp_start::com::uart;
use esp_start::compass::{CompassCalibration, CompassKind, MagneticField};
use esp_start::io::{self, ButtonId, InputEvent, ScreenOutPins, SdOutPins};
use esp_start::leds::{LedController, UPDATE_INTERVAL_MS};
use esp_start::net::HttpClient;
use esp_start::screen::{DEFAULT_CONTRAST, ScreenController, ScreenDriver};
use esp_start::sd::{SdStorage, SdStorageError};
use esp_start::wifi::WifiNetwork;
use esp_start::{ai, runtime, spi_bus, timer};

const WIFI_SSID: Option<&str> = option_env!("ESP_START_WIFI_SSID");
const WIFI_PASSWORD: Option<&str> = option_env!("ESP_START_WIFI_PASSWORD");
const SERVER_ADDR_IPPORT: (Ipv4Addr, u16) = (Ipv4Addr::new(192, 168, 1, 102), 1234);
const BLE_LOGGING_ENABLED: bool = false;

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

    // --- IO ---
    io::setup(peripherals.IO_MUX);
    io::setup_backlight_button(peripherals.GPIO32);
    io::setup_led_mode_button(peripherals.GPIO27);
    uart.write_str("[INPUT] Backlight button ready\r\n").unwrap();
    uart.write_str("[INPUT] LED mode button ready\r\n").unwrap();

    // --- Scheduling ---
    let timer_group = TimerGroup::new(peripherals.TIMG0);
    timer::setup(timer_group.timer0, UPDATE_INTERVAL_MS);
    runtime::setup_scheduler(peripherals.FROM_CPU_INTR0, timer_group.timer1);

    // --- LEDs ---
    // GPIO25 blinks. GPIO26 and GPIO14 are the complementary PWM fade pair.
    let mut leds = LedController::new(
        peripherals.LEDC,
        peripherals.GPIO25,
        peripherals.GPIO26,
        peripherals.GPIO14,
    );

    // --- Bluetooth (BLE) ---
    uart.write_str("[BLE] Initializing ... ").unwrap();
    let mut bluetooth = BluetoothManager::init(peripherals.BT, &spawner);
    uart.write_str("SUCCESS\r\n").unwrap();

    // --- Shared SPI bus ---
    let sd_out_pins = SdOutPins::new(peripherals.GPIO16);
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
    // embedded-sdmmc leaves the >=74 CS-high startup clocks to the caller.
    // Both devices are deselected here; 0xFF keeps MOSI high for 80 clocks.
    {
        let mut bus = spi_bus.borrow_mut();
        bus.write(&[0xFF; 10]).expect("SD startup clocks failed");
        bus.flush().expect("SD startup clock flush failed");
    }
    let sd_spi = RefCellDevice::new(&spi_bus, sd_out_pins.cs, Delay::new()).unwrap();
    let storage = SdStorage::new(sd_spi, Delay::new());

    match storage.mount(&mut uart) {
        Ok(sd) => {
            uart.write_str("SUCCESS\r\n").unwrap();
            let size_mb = sd.size_bytes() / 1_048_576;
            writeln!(uart, "[SD] Inserted; Capacity: {size_mb} MB").unwrap();

            if let Err(error) = sd.for_each_root_entry(|entry| {
                writeln!(
                    uart,
                    "{} {:?} {} bytes",
                    entry.name, entry.attributes, entry.size
                ).unwrap();
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
        }
        Err(error) => {
            uart.write_str("FAILED\r\n").unwrap();
            writeln!(uart, "--> Error: {error:?}").unwrap();
        }
    }

    // --- Wi-Fi and HTTP ---
    let wifi_network = match WIFI_SSID {
        Some(ssid) => {
            WifiNetwork::connect(peripherals.WIFI, &spawner, ssid, WIFI_PASSWORD, &mut uart).await
        }
        None => {
            uart.write_str("[WIFI] Skipped: ESP_START_WIFI_SSID is not set\r\n").unwrap();
            None
        }
    };
    let mut http_client = match wifi_network.as_ref() {
        Some(network) => Some(HttpClient::new(network.stack(), SERVER_ADDR_IPPORT)),
        None => None,
    };

    // --- Pulse counter on GPIO33 ---
    let pulse_counter = Pcnt::new(peripherals.PCNT);
    let pulse_unit = pulse_counter.unit0;
    // pcnt::setup(
    //     &pulse_unit.channel0,
    //     peripherals.GPIO33,
    //     PinConfig::PullUp.as_input(),
    //     EdgeMode::Increment,
    //     EdgeMode::Hold,
    // );
    // pulse_unit.clear();
    // pulse_unit.resume();
    let mut last_pulse_count = pulse_unit.value();

    // --- Compass ---
    let mut compass_i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(100)),
    ).unwrap().with_sda(peripherals.GPIO13).with_scl(peripherals.GPIO33);

    let compass = CompassKind::detect(&mut compass_i2c);
    match compass {
        Some(kind) => {
            writeln!(uart, "[COMPASS] Detected: {kind:?}").unwrap();

            if let Err(err) = kind.init(&mut compass_i2c) {
                writeln!(uart, "[COMPASS] Init failed: {err:?}").unwrap();
            }
        }
        None => {
            writeln!(uart, "[COMPASS] No supported sensor detected").unwrap();
        }
    }

    let mut calibration: Option<CompassCalibration> = None;
    let mut next_compass_read = Instant::now();
    let mut next_greeting_text = Instant::now() + Duration::from_secs(10);

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

        while let Some(event) = bluetooth.next_event() {
            match event {
                BluetoothEvent::Connected => {
                    writeln!(uart, "[BLE] Connected").unwrap();
                }
                BluetoothEvent::Disconnected => {
                    writeln!(uart, "[BLE] Disconnected").unwrap();
                }
                BluetoothEvent::DataReceived { len, data } => {
                    let payload = &data[..len];
                    if let Ok(text) = str::from_utf8(payload) {
                        writeln!(uart, "[BLE] RX: {text}").unwrap();
                    } else {
                        writeln!(uart, "[BLE] RX bytes: {:02x?}", payload).unwrap();
                    }
                }
                BluetoothEvent::ReadRequested => {
                    writeln!(uart, "[BLE] REad Requested").unwrap();
                }
                BluetoothEvent::StatusReadRequested => {
                    writeln!(uart, "[BLE] Status Read Requested").unwrap();
                }
            }
        }

        if BLE_LOGGING_ENABLED && bluetooth.poll() {
            writeln!(
                uart,
                "[BLE] Discovered Devices: {:?}\r\n",
                bluetooth.devices()
            ).unwrap();
        }

        leds.update();

        if Instant::now() >= next_compass_read {
            if let Some(kind) = compass {
                match kind.read(&mut compass_i2c) {
                    Ok(field) => {
                        let cal = calibration.get_or_insert_with(|| {
                            CompassCalibration::new(field)
                        });

                        cal.update(field);

                        let (ox, oy, oz) = cal.offset();

                        writeln!(
                            uart,
                            "[COMPASS] X={} Y={} Z={}",
                            field.x, field.y, field.z
                        ).unwrap();

                        writeln!(
                            uart,
                            "[CAL] MIN=({},{},{}) MAX=({},{},{})",
                            cal.min.x, cal.min.y, cal.min.z,
                            cal.max.x, cal.max.y, cal.max.z
                        ).unwrap();

                        writeln!(
                            uart,
                            "[CAL] OFFSET=({:.1},{:.1},{:.1})",
                            ox, oy, oz
                        ).unwrap();

                        let final_field = MagneticField {
                            x: ((field.x as f32) - ox) as i16,
                            y: ((field.y as f32) - oy) as i16,
                            z: ((field.z as f32) - oz) as i16,
                        };
                        // Screen Test
                        screen.clear();
                        screen.draw_fmt((0, 10), format_args!("X:{:.1}\r\nY:{:.1}\r\nZ:{:.1}", final_field.x, final_field.y, final_field.z), true, false);
                        screen.flush().unwrap();
                    }
                    Err(err) => {
                        writeln!(uart, "[COMPASS] Read error: {err:?}").unwrap();
                    }
                }
            }

            next_compass_read = Instant::now() + Duration::from_millis(500);
        }

        let pulse_count = pulse_unit.value();
        if pulse_count != last_pulse_count {
            writeln!(uart, "[PCNT] Current value: {pulse_count}").unwrap();
            last_pulse_count = pulse_count;
        }

        // --- AI Chatting ---
        if http_client.is_some() && Instant::now() >= next_greeting_text {
            generate_greeting_msg(&mut uart, &mut screen, &mut http_client).await;
            next_greeting_text += Duration::from_secs(10);
        }

        Timer::after(Duration::from_millis(1)).await;
    }
}

async fn generate_greeting_msg(
    uart: &mut Uart<'_, Blocking>,
    screen: &mut ScreenController<RefCellDevice<'_, Spi<'_, Blocking>, Output<'_>, Delay>>,
    http_client: &mut Option<HttpClient>,
) {
    uart.write_str("\r\n").unwrap();
    uart.write_str("Chatting with first model...\r\n").unwrap();
    screen.clear();
    screen.draw_text((0, 0), "Chatting ...", true, false);

    let chat_message = AiChatRequest {
        selected_model_name: ai::DEFAULT_MODEL_KEY.to_string(),
        input: "Hi bro! I am Saleh. This is the new first-time attraction with you.".to_string(),
        system_prompt: None,
    };

    if let Some(client) = http_client.as_mut() {
        AiClient::send_message(client, chat_message, |result| match result {
            Ok(response) => {
                writeln!(uart, "[AI] SendMessage: message processing ...").unwrap();
                for output in &response.output {
                    if output.output_type == "message" {
                        writeln!(
                            uart,
                            "[AI] SendMessage: Received message: {}",
                            output.content
                        ).unwrap();
                        screen.clear();
                        screen.draw_fmt((0, 0), format_args!("{}", output.content), true, false);
                    }
                }
            }
            Err(error) => match error {
                AiError::Http(error) => {
                    writeln!(uart, "[AI] SendMessage: Request send failed: {error:?}").unwrap();
                }
                AiError::HttpStatus(error_code) => {
                    writeln!(
                        uart,
                        "[AI] SendMessage: Request had an error: {error_code:?}"
                    ).unwrap();
                }
                AiError::Json(error) => {
                    writeln!(uart, "[AI] SendMessage: Request parse failed: {error:?}").unwrap();
                }
            },
        }).await;
    }
    if let Err(error) = screen.flush() {
        writeln!(uart, "[LCD] Flush failed: {error:#?}").unwrap();
    }
}
