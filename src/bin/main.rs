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
use esp_hal::ledc::{channel::Number as ChannelNumber, timer::Number as TimerNumber};
use esp_hal::uart::Uart;
use esp_hal::{Blocking, main};
use esp_start::com::uart;
use esp_start::io::OutputPins;
use esp_start::pwm::PwmConfig;
use esp_start::utils::delay;
use esp_start::{io, pwm, timer};

const DEBOUNCE_DELAY: u64 = 10;
const TIMER_DELAY: u64 = 300;

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

    let mut pwm_control = pwm::setup(
        _peripherals.LEDC,
        _peripherals.GPIO19,
        TimerNumber::Timer0,
        ChannelNumber::Channel1,
        PwmConfig::led_default(),
    );

    io::setup(_peripherals.IO_MUX, _peripherals.GPIO22);
    timer::setup(_peripherals.TIMG0, TIMER_DELAY);

    ////////

    let mut last_event_call_count = 0;

    let mut fade_mulp = 1;
    let mut down = false;
    loop {
        blink_led(&mut uart, &mut output_pins, &mut last_event_call_count);
        control_led_if_button_pressed(&mut output_pins);

        pwm_control.set_duty(fade_mulp * 10, 100);
        if down {
            if fade_mulp < 1 {
                down = false;
                continue;
            }
            fade_mulp -= 1;
        } else {
            if fade_mulp >= 10 {
                down = true;
                continue;
            }
            fade_mulp += 1;
        }

        delay(DEBOUNCE_DELAY);
    }
}

// #####################

fn control_led_if_button_pressed(output_pins: &mut OutputPins) {
    if io::test_button_pressed() {
        output_pins.test_led.set_high();
    } else {
        output_pins.test_led.set_low();
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
