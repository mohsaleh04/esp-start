#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::fmt::Write;
use core::net::Ipv4Addr;
use core::panic::PanicInfo;
use embassy_executor::Spawner;
use embassy_net::IpAddress;
use embassy_net::tcp::State;
use esp_hal::Blocking;
use esp_hal::clock::CpuClock;
use esp_hal::ledc::timer::Timer;
use esp_hal::ledc::{
    Ledc, LowSpeed, channel::Number as ChannelNumber, timer::Number as TimerNumber,
};
use esp_hal::pcnt::Pcnt;
use esp_hal::pcnt::channel::EdgeMode;
use esp_hal::time::Instant;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::Uart;
use esp_start::com::uart;
use esp_start::io::{OutputPins, PinConfig};
use esp_start::net::socket;
use esp_start::pwm::{PwmChannelConfig, PwmTimerConfig};
use esp_start::{io, net, pcnt, pwm, runtime, timer, wifi};
use static_cell::StaticCell;

const DEBOUNCE_DURATION_MS: u64 = 700;
const TIMER_DELAY_MS: u64 = 300;
const WIFI_SSID: &str = "HomeADSL";
const WIFI_PASSWORD: &str = "Home#1405";

static PWM_TIMER: StaticCell<Timer<'static, LowSpeed>> = StaticCell::new();

enum LedMode {
    Blink,
    Fade,
    Off,
}

// #############

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut uart = uart::setup(peripherals.UART0, peripherals.GPIO1, peripherals.GPIO3);
    uart.write_str("PANIIIIIIC!!!!\r\n\r\n").expect("failed uart panic!");
    loop {}
}

// Don't Remove This Code! Code for bootloader
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    runtime::allocate_heap();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut uart = uart::setup(peripherals.UART0, peripherals.GPIO1, peripherals.GPIO3);
    let mut output_pins = OutputPins::new(peripherals.GPIO23, peripherals.GPIO21);

    // === PWM | LEDC ===
    let mut ledc = Ledc::new(peripherals.LEDC);
    let pwm_timer = PWM_TIMER.init(ledc.timer::<LowSpeed>(TimerNumber::Timer0));
    pwm::setup_timer(pwm_timer, PwmTimerConfig::default(500));

    let mut pwm_control = pwm::setup_channel(
        &mut ledc,
        peripherals.GPIO19,
        pwm_timer,
        ChannelNumber::Channel1,
        PwmChannelConfig::default(),
    );

    let mut pwm_control2 = pwm::setup_channel(
        &mut ledc,
        peripherals.GPIO18,
        pwm_timer,
        ChannelNumber::Channel2,
        PwmChannelConfig::default(),
    );

    // === Interrupts (IO/Timer) ===
    io::setup(peripherals.IO_MUX, peripherals.GPIO22);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    timer::setup(timg0.timer0, TIMER_DELAY_MS);
    runtime::setup_scheduler(peripherals.SW_INTERRUPT, timg0.timer1);

    // === WIFI ===
    let (mut wifi_controller, wifi_interfaces) = wifi::setup(peripherals.WIFI);
    let station = wifi_interfaces.station;
    write!(uart, "wifi module init\r\n").unwrap();

    uart.write_str("connecting to wifi ...\r\n").unwrap();
    wifi::config::set_station_config(&mut wifi_controller, WIFI_SSID, Some(WIFI_PASSWORD));

    uart.write_str("network stack init\r\n").unwrap();
    let (net_stack, net_runner) = net::setup(station);
    net::runner::run_wifi_net_task(spawner, net_runner);

    let wifi_connection_success = wifi::connection::connect(&mut wifi_controller, &mut uart).await;

    if wifi_connection_success {
        net::wait_for_config_up(net_stack, &mut uart).await;

        uart.write_str("Network is UP \r\n").unwrap();
        if let Some(config) = net_stack.config_v4() {
            write!(uart, "\tIPv4 config: {:?}\r\n", config).unwrap();
        }
    }

    let mut conn_socket = socket::new(net_stack);
    let mut tcp_socket_up = false;
    if wifi_connection_success && net_stack.is_config_up() {
        // Create TCP Socket
        uart.write_str("Creating a TCP Connection to your laptop...\r\n")
            .unwrap();
        socket::connect_tcp(
            &mut conn_socket,
            IpAddress::Ipv4(Ipv4Addr::new(192, 168, 1, 101)),
            80,
        ).await;
        if conn_socket.state() == State::Established {
            uart.write_str("connection applied\r\n").unwrap();
            tcp_socket_up = true;
        } else {
            write!(uart, "failed to connect socket: {:?}", conn_socket.state()).unwrap();
        }

        // Request
        let test_request =
            b"GET /gpio/1 HTTP/1.1\r\n\
            Host: 192.168.1.101\r\n\
            \r\n";
        conn_socket
            .write(test_request)
            .await
            .expect("write request failed");
    }

    // === PCNT (Pulse Counter) ===
    let pcnt = Pcnt::new(peripherals.PCNT);
    let unit = pcnt.unit0;
    pcnt::setup(
        &unit.channel0,
        peripherals.GPIO33,
        PinConfig::PullUp.as_input(),
        EdgeMode::Increment,
        EdgeMode::Hold,
    );

    unit.clear();
    unit.resume();

    ////////

    let mut last_event_call_count = 0;

    let mut led_mod = LedMode::Off;
    let mut pwm_led_fade_mulp = 1;
    let mut pwm_led_fade_down = false;

    let mut last_button_pressed: Option<Instant> = None;

    let mut last_pcnt_count = 0;

    let mut test_response_buffer = [0u8; 512];

    loop {
        let mut button_act_permitted = true;
        if io::test_button_pressed() {
            if let Some(last_btn_act) = last_button_pressed.as_mut() {
                if (Instant::now().duration_since_epoch().as_millis()
                    - last_btn_act.duration_since_epoch().as_millis())
                    <= DEBOUNCE_DURATION_MS
                {
                    button_act_permitted = false;
                }
            }

            if button_act_permitted {
                led_mod = switch_led_mode(led_mod);
                last_button_pressed = Some(Instant::now());

                pwm_control.off();
                pwm_control2.off();
                output_pins.blink_led.set_low();
                output_pins.test_led.set_high();
            }
        } else {
            output_pins.test_led.set_low();
        }

        // --- LED Handler Menu ---
        match led_mod {
            LedMode::Blink => {
                blink_led(&mut uart, &mut output_pins, &mut last_event_call_count);
            }
            LedMode::Fade => {
                // uart.write_str("pwm leds selected.\r\n").unwrap();
                let upper_bound = 10;
                pwm_control.set_duty(pwm_led_fade_mulp * 10, 100);
                pwm_control2.set_duty((upper_bound - pwm_led_fade_mulp) * 10, 100);
                if pwm_led_fade_down {
                    if pwm_led_fade_mulp < 1 {
                        pwm_led_fade_down = false;
                        pwm_led_fade_mulp = 0;
                        continue;
                    }
                    pwm_led_fade_mulp -= 1;
                } else {
                    if pwm_led_fade_mulp >= upper_bound {
                        pwm_led_fade_down = true;
                        pwm_led_fade_mulp = upper_bound;
                        continue;
                    }
                    pwm_led_fade_mulp += 1;
                }
            }
            LedMode::Off => {
                output_pins.test_led.set_low();
                output_pins.blink_led.set_low();
                pwm_control.off();
                pwm_control2.off();
            }
        }

        // TODO: Needed an extra push button in _GPIO33_ for create this PCNT pulse  (Even though could be a clock generator like NE555P circuit.)
        let count = &unit.value();
        if last_pcnt_count != *count {
            write!(uart, "current pcnt value: {}\r\n", count).unwrap();
            last_pcnt_count = *count;
        }

        // ======== WIFI =========
        if tcp_socket_up {
            write!(
                uart,
                "\t- Current socket status: {}\r\n\r\n",
                conn_socket.state()
            ).unwrap();
            match conn_socket.read(&mut test_response_buffer).await {
                Ok(0) => {
                    uart.write_str("TCP Connection Closed\r\n").unwrap();
                    tcp_socket_up = false;
                }
                Ok(size) => {
                    if let Ok(text) = core::str::from_utf8(&test_response_buffer[..size]) {
                        write!(uart, "[WIFI] {}\r\n", text).unwrap();
                    } else {
                        uart.write_str("Unparsable data received!").unwrap();
                    }
                }
                Err(error) => {
                    write!(uart, "[WIFI] Failed to receive data: {:?}", error).unwrap();
                    tcp_socket_up = false;
                }
            }
        }
    }
}

// #####################

fn switch_led_mode(led_mod: LedMode) -> LedMode {
    match led_mod {
        LedMode::Off => LedMode::Blink,
        LedMode::Blink => LedMode::Fade,
        LedMode::Fade => LedMode::Off,
    }
}

fn blink_led(
    uart: &mut Uart<Blocking>,
    board_pins: &mut OutputPins,
    last_event_call_count: &mut u32,
) {
    let timer_counter = timer::event_counter();
    let mut i = 0;
    while i < timer_counter.wrapping_sub(*last_event_call_count) {
        write!(uart, "L{}: toggle led!\r\n", *last_event_call_count + i + 1).unwrap();
        board_pins.blink_led.toggle();
        i += 1
    }
    *last_event_call_count = timer_counter;
}
