use crate::timer::handler::timer_handler;
use crate::timer::scheduler::schedule_timer;
use core::sync::atomic::Ordering;
use esp_hal::timer::PeriodicTimer;
use esp_hal::timer::timg::{Timer};

mod handler;
mod scheduler;
mod states;

pub fn setup(timer: Timer<'static>, timer_every_millis: u64) {
    let mut timer = PeriodicTimer::new(timer);
    timer.set_interrupt_handler(timer_handler);

    schedule_timer(timer, timer_every_millis);
}

pub fn event_counter() -> u32 {
    states::TIMER_COUNTER.load(Ordering::Relaxed)
}
