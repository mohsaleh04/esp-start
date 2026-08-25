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
use esp_hal::ledc::timer::Timer;
use esp_hal::ledc::{
    channel::Number as ChannelNumber, timer::Number as TimerNumber, Ledc, LowSpeed,
};
use esp_hal::time::Instant;
use esp_hal::uart::Uart;
use esp_hal::{Blocking, main};
use esp_hal::gpio::{Input, InputConfig};
use esp_hal::pcnt::channel::EdgeMode;
use esp_hal::pcnt::Pcnt;
use esp_start::com::uart;
use esp_start::io::{OutputPins, PinConfig};
use esp_start::utils::delay;
use esp_start::{io, pwm, timer};
use static_cell::StaticCell;

const DEBOUNCE_DELAY: u64 = 10;
const TIMER_DELAY: u64 = 300;
static PWM_TIMER: StaticCell<Timer<'static, LowSpeed>> = StaticCell::new();

enum LedMode {
    Blink,
    Fade,
    Off
}

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
    let mut output_pins = OutputPins::new(_peripherals.GPIO23, _peripherals.GPIO21);

    let mut ledc = Ledc::new(_peripherals.LEDC);
    let pwm_timer = PWM_TIMER.init(ledc.timer::<LowSpeed>(TimerNumber::Timer0));
    pwm::setup_timer(pwm_timer, PwmTimerConfig::default(500));

    let mut pwm_control = pwm::setup_channel(
        &mut ledc,
        _peripherals.GPIO19,
        pwm_timer,
        ChannelNumber::Channel1,
        PwmChannelConfig::default(),
    );

    let mut pwm_control2 = pwm::setup_channel(
        &mut ledc,
        _peripherals.GPIO18,
        pwm_timer,
        ChannelNumber::Channel2,
        PwmChannelConfig::default(),
    );

    io::setup(_peripherals.IO_MUX, _peripherals.GPIO22);
    timer::setup(_peripherals.TIMG0, TIMER_DELAY);

    ////////

    let pcnt = Pcnt::new(_peripherals.PCNT);

    let unit = pcnt.unit0;
    let input = Input::new(
        _peripherals.GPIO33,
        PinConfig::PullUp.as_input(),
    );
    let signal = input.peripheral_input();

    let channel = &unit.channel0;

    channel.set_edge_signal(signal);

    channel.set_input_mode(
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
    loop {
        let mut button_act_permitted = true;
        if io::test_button_pressed() {
            if let Some(last_btn_act) = last_button_pressed.as_mut() {
                if (Instant::now().duration_since_epoch().as_millis()
                      - last_btn_act.duration_since_epoch().as_millis())
                    <= 700 {
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

        // Handler Menu
        match led_mod {
            LedMode::Blink => {
                blink_led(&mut uart, &mut output_pins, &mut last_event_call_count);

                // TODO: Needed an extra push button in _GPIO33_ for create this PCNT pulse  (Eventhough could be a clock generator like NE555P cercuit.)
                let count = &unit.value();
                write!(uart, "current pcnt value: {}\r\n", count).unwrap();
            },
            LedMode::Fade => {
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
            },
            LedMode::Off => {
                output_pins.test_led.set_low();
                output_pins.blink_led.set_low();
                pwm_control.off();
                pwm_control2.off();
            }
        }
      
        delay(DEBOUNCE_DELAY);
    }
}

// #####################

fn switch_led_mode(led_mod: LedMode) -> LedMode {
    match led_mod {
        LedMode::Off => LedMode::Blink,
        LedMode::Blink => LedMode::Fade,
        LedMode::Fade => LedMode::Off
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
