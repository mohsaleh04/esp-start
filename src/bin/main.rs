#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use critical_section::Mutex;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Event, Input, InputConfig, Io, Level, Output, OutputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::time::Duration;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::timer::PeriodicTimer;
use esp_hal::uart::{Config, Uart};
use esp_hal::{handler, main, ram};
use esp_start::delay;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

// Don't Remove This Code! Code for bootloader
esp_bootloader_esp_idf::esp_app_desc!();

const TIMER_DELAY: u64 = 300;

static TIMER_COUNTER: AtomicU32 = AtomicU32::new(0);
static TIMER: Mutex<RefCell<Option<PeriodicTimer<'static, esp_hal::Blocking>>>> =
    Mutex::new(RefCell::new(None));

static TEST_BTN: Mutex<RefCell<Option<Input<'static>>>> = Mutex::new(RefCell::new(None));
static TEST_BTN_PRESSED: AtomicBool = AtomicBool::new(false);

#[handler]
fn timer_handler() {
    critical_section::with(|cs| {
        let mut timer = TIMER.borrow_ref_mut(cs);
        if let Some(timer) = timer.as_mut() {
            timer.clear_interrupt();
        }
    });

    TIMER_COUNTER.fetch_add(1, Ordering::Relaxed);
}

#[handler]
fn gpio_handler() {
     critical_section::with(|cs| {
        let mut btn = TEST_BTN.borrow_ref_mut(cs);
        if let Some(btn) = btn.as_mut() {
            btn.clear_interrupt();
            TEST_BTN_PRESSED.store(btn.is_low(), Ordering::Relaxed);
        }
    });
}

// ######################

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    // esp_alloc::heap_allocator!(size: 36 * 1024);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let pullup_btn_cfg = InputConfig::default().with_pull(Pull::Up);

    let _peripherals = esp_hal::init(config);

    let mut uart = Uart::new(_peripherals.UART0, Config::default())
        .unwrap()
        .with_tx(_peripherals.GPIO1)
        .with_rx(_peripherals.GPIO3);

    let mut led = Output::new(_peripherals.GPIO23, Level::Low, OutputConfig::default());
    let mut btn_led = Output::new(_peripherals.GPIO21, Level::Low, OutputConfig::default());
    let test_btn = Input::new(_peripherals.GPIO22, pullup_btn_cfg);

    let mut io = Io::new(_peripherals.IO_MUX);
    io.set_interrupt_handler(gpio_handler);

    critical_section::with(|cs| {
        TEST_BTN.borrow_ref_mut(cs).replace(test_btn);
    });
    critical_section::with(|cs| {
        let mut this_btn = TEST_BTN.borrow_ref_mut(cs);
        let btn = this_btn.as_mut().unwrap();

        btn.listen(Event::AnyEdge);
    });

    uart.write_str("Setup timer ...\r\n").unwrap();
    // let sw_interrupt = SoftwareInterruptControl::new(_peripherals.SW_INTERRUPT);

    let timg0 = TimerGroup::new(_peripherals.TIMG0);
    let mut timer = PeriodicTimer::new(timg0.timer0);
    timer.set_interrupt_handler(timer_handler);

    critical_section::with(|cs| {
        TIMER.borrow_ref_mut(cs).replace(timer);
    });
    critical_section::with(|cs| {
        let mut timer = TIMER.borrow_ref_mut(cs);
        let timer = timer.as_mut().unwrap();

        timer.start(Duration::from_millis(TIMER_DELAY)).unwrap();
        timer.listen();
    });

    // esp_rtos::start(timg0.timer1, sw_interrupt.software_interrupt0);

    // uart.write_str("Prepare wifi ...").unwrap();
    // let mut wifi = match WifiController::new(_peripherals.WIFI, Default::default()) {
    //     Ok(wifi) => {
    //         uart.write_str("WiFi controller initialized!\r\n").unwrap();
    //         wifi
    //     }
    //     Err(_) => {
    //         uart.write_str("WiFi init FAILED!\r\n").unwrap();
    //
    //         // ERR LED Blinking
    //         TIMER_DELAY.store(150, Ordering::Relaxed);
    //         loop {
    //             if TIMER_FIRED.swap(false, Ordering::Relaxed) {
    //                 uart.write_str("toggle led!\r\n").unwrap();
    //                 led.toggle();
    //             }
    //         }
    //     }
    // };

    let mut last_event_call_count = 0;
    loop {
        let timer_counter = TIMER_COUNTER.load(Ordering::Relaxed);
        let mut i = 0;
        while i < timer_counter.wrapping_sub(last_event_call_count) {
            write!(uart, "L{}: toggle led!\r\n", last_event_call_count + i + 1).unwrap();
            led.toggle();
            i += 1
        }
        last_event_call_count = timer_counter;

        let test_btn_pressed = TEST_BTN_PRESSED.load(Ordering::Relaxed);
        if test_btn_pressed {
            btn_led.set_high();
        } else {
            btn_led.set_low();
        }
        delay(10);
    }
}
