#![no_std]
#![no_main]

use core::fmt::Write;
use core::hint::spin_loop;
use core::net::Ipv4Addr;
use core::ops::ControlFlow;
use core::panic::PanicInfo;

use embassy_executor::Spawner;
use embassy_net::IpAddress;
use embassy_net::tcp::State;
use embassy_time::{Duration as EmbassyDuration, Timer as EmbassyTimer, with_timeout};
use embedded_hal_bus::spi::RefCellDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::ledc::timer::Timer;
use esp_hal::ledc::{
    Ledc, LowSpeed, channel::Number as ChannelNumber, timer::Number as TimerNumber,
};
use esp_hal::pcnt::Pcnt;
use esp_hal::pcnt::channel::EdgeMode;
use esp_hal::timer::timg::TimerGroup;
use esp_start::com::uart;
use esp_start::io::{self, ButtonId, InputEvent, PinConfig, ScreenOutPins, SdOutPins};
use esp_start::net::socket;
use esp_start::pwm::{PwmChannelConfig, PwmTimerConfig};
use esp_start::screen::{DEFAULT_CONTRAST, ScreenController, ScreenDriver};
use esp_start::sd::{SdStorage, SdStorageError};
use esp_start::{net, pcnt, pwm, runtime, spi_bus, timer, wifi};
use static_cell::StaticCell;

const TIMER_DELAY_MS: u64 = 300;
const NETWORK_POLL_MS: u64 = 10;
const WIFI_SSID: &str = "HomeADSL";
const WIFI_PASSWORD: &str = "Home#1405";
const SERVER_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 101);
const SERVER_PORT: u16 = 80;

static PWM_TIMER: StaticCell<Timer<'static, LowSpeed>> = StaticCell::new();

#[derive(Clone, Copy, Debug)]
enum LedMode {
    Off,
    Blink,
    Fade,
}

impl LedMode {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Blink,
            Self::Blink => Self::Fade,
            Self::Fade => Self::Off,
        }
    }
}

struct FadeState {
    duty: u8,
    descending: bool,
    last_timer_count: u32,
}

impl FadeState {
    fn new(timer_count: u32) -> Self {
        Self {
            duty: 0,
            descending: false,
            last_timer_count: timer_count,
        }
    }

    fn reset(&mut self, timer_count: u32) {
        self.duty = 0;
        self.descending = false;
        self.last_timer_count = timer_count;
    }

    fn advance(&mut self) {
        if self.descending {
            if self.duty == 0 {
                self.descending = false;
                self.duty = 10;
            } else {
                self.duty = self.duty.saturating_sub(10);
            }
        } else if self.duty >= 100 {
            self.descending = true;
            self.duty = 90;
        } else {
            self.duty += 10;
        }
    }
}

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

    // --- Input and periodic timer ---
    io::setup(peripherals.IO_MUX);
    io::setup_primary_button(peripherals.GPIO32);
    uart.write_str("[INPUT] Primary button ready on GPIO32\r\n")
        .unwrap();

    let timer_group = TimerGroup::new(peripherals.TIMG0);
    timer::setup(timer_group.timer0, TIMER_DELAY_MS);
    runtime::setup_scheduler(peripherals.SW_INTERRUPT, timer_group.timer1);

    // --- LEDs: GPIO25 blink, GPIO26/GPIO27 complementary PWM fade ---
    let mut blink_led = Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default());

    let mut ledc = Ledc::new(peripherals.LEDC);
    let pwm_timer = PWM_TIMER.init(ledc.timer::<LowSpeed>(TimerNumber::Timer0));
    pwm::setup_timer(pwm_timer, PwmTimerConfig::default(500));
    let mut fade_led_a = pwm::setup_channel(
        &mut ledc,
        peripherals.GPIO26,
        pwm_timer,
        ChannelNumber::Channel1,
        PwmChannelConfig::default(),
    );
    let mut fade_led_b = pwm::setup_channel(
        &mut ledc,
        peripherals.GPIO27,
        pwm_timer,
        ChannelNumber::Channel2,
        PwmChannelConfig::default(),
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

    // --- Wi-Fi and TCP ---
    uart.write_str("[WIFI] Initializing ... ").unwrap();
    let (mut wifi_controller, wifi_interfaces) = wifi::setup(peripherals.WIFI);
    wifi::config::set_station_config(&mut wifi_controller, WIFI_SSID, Some(WIFI_PASSWORD));
    let (net_stack, net_runner) = net::setup(wifi_interfaces.station);
    net::runner::run_wifi_net_task(spawner, net_runner);
    uart.write_str("SUCCESS\r\n[WIFI] Connecting ...\r\n")
        .unwrap();

    let wifi_connected = wifi::connection::connect(&mut wifi_controller, &mut uart).await;
    if wifi_connected {
        net::wait_for_config_up(net_stack, &mut uart).await;
        uart.write_str("[NET] Network is up\r\n").unwrap();
        if let Some(config) = net_stack.config_v4() {
            writeln!(uart, "[NET] IPv4 config: {config:?}").unwrap();
        }
    }

    let mut tcp_socket = socket::new(net_stack);
    let mut tcp_socket_up = false;
    if wifi_connected && net_stack.is_config_up() {
        writeln!(uart, "[TCP] Connecting to {SERVER_IP}:{SERVER_PORT} ...").unwrap();
        let connect_result =
            socket::connect_tcp(&mut tcp_socket, IpAddress::Ipv4(SERVER_IP), SERVER_PORT).await;

        if let Err(error) = connect_result {
            writeln!(uart, "[TCP] Connection failed: {error:?}").unwrap();
        } else if tcp_socket.state() == State::Established {
            tcp_socket_up = true;
            uart.write_str("[TCP] Connected\r\n").unwrap();
            let request = b"GET /gpio/1 HTTP/1.1\r\n\
                Host: 192.168.1.101\r\n\
                Connection: close\r\n\
                \r\n";
            if let Err(error) = tcp_socket.write(request).await {
                writeln!(uart, "[TCP] Request failed: {error:?}").unwrap();
                tcp_socket_up = false;
            }
        } else {
            writeln!(uart, "[TCP] Connection failed: {:?}", tcp_socket.state()).unwrap();
        }
    }

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

    let mut led_mode = LedMode::Off;
    let mut last_blink_timer_count = timer::event_counter();
    let mut fade_state = FadeState::new(last_blink_timer_count);
    let mut last_pulse_count = pulse_unit.value();
    let mut response_buffer = [0_u8; 512];

    loop {
        while let Some(event) = io::next_input_event() {
            match event {
                InputEvent::ButtonPressed(ButtonId::Primary) => {
                    led_mode = led_mode.next();
                    writeln!(uart, "[INPUT] LED mode: {led_mode:?}").unwrap();

                    blink_led.set_low();
                    fade_led_a.off();
                    fade_led_b.off();
                    let timer_count = timer::event_counter();
                    last_blink_timer_count = timer_count;
                    fade_state.reset(timer_count);
                }
                InputEvent::ButtonReleased(ButtonId::Primary) => {
                    uart.write_str("[INPUT] Primary button released\r\n")
                        .unwrap();
                }
            }
        }

        let timer_count = timer::event_counter();
        match led_mode {
            LedMode::Off => {}
            LedMode::Blink => {
                let elapsed_ticks = timer_count.wrapping_sub(last_blink_timer_count);
                if elapsed_ticks != 0 {
                    if !elapsed_ticks.is_multiple_of(2) {
                        blink_led.toggle();
                    }
                    last_blink_timer_count = timer_count;
                }
            }
            LedMode::Fade => {
                let elapsed_ticks = timer_count.wrapping_sub(fade_state.last_timer_count);
                for _ in 0..elapsed_ticks {
                    fade_state.advance();
                }
                if elapsed_ticks != 0 {
                    fade_led_a.set_duty(fade_state.duty);
                    fade_led_b.set_duty(100 - fade_state.duty);
                    fade_state.last_timer_count = timer_count;
                }
            }
        }

        let pulse_count = pulse_unit.value();
        if pulse_count != last_pulse_count {
            writeln!(uart, "[PCNT] Current value: {pulse_count}").unwrap();
            last_pulse_count = pulse_count;
        }

        if tcp_socket_up {
            match with_timeout(
                EmbassyDuration::from_millis(NETWORK_POLL_MS),
                tcp_socket.read(&mut response_buffer),
            )
            .await
            {
                Ok(Ok(0)) => {
                    uart.write_str("[TCP] Connection closed\r\n").unwrap();
                    tcp_socket_up = false;
                }
                Ok(Ok(size)) => match core::str::from_utf8(&response_buffer[..size]) {
                    Ok(text) => writeln!(uart, "[TCP] {text}").unwrap(),
                    Err(_) => uart.write_str("[TCP] Received non-UTF-8 data\r\n").unwrap(),
                },
                Ok(Err(error)) => {
                    writeln!(uart, "[TCP] Read failed: {error:?}").unwrap();
                    tcp_socket_up = false;
                }
                Err(_) => {}
            }
        }

        // Yield even when the TCP socket is down so the network runner can execute.
        EmbassyTimer::after(EmbassyDuration::from_millis(1)).await;
    }
}
