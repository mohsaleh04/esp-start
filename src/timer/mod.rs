use crate::timer::handler::timer_handler;
use crate::timer::scheduler::schedule_timer;
use core::sync::atomic::Ordering;
use esp_hal::timer::timg::{TimerGroup, TimerGroupInstance};
use esp_hal::timer::PeriodicTimer;

mod scheduler;
mod handler;
mod states;

pub fn setup<T: TimerGroupInstance + 'static>(_timer_group_perip: T, timer_every_millis: u64) {
    let timg0 = TimerGroup::new(_timer_group_perip);
    let mut timer = PeriodicTimer::new(timg0.timer0);
    timer.set_interrupt_handler(timer_handler);

    schedule_timer(timer, timer_every_millis);
}

pub fn event_counter() -> u32 {
    states::TIMER_COUNTER.load(Ordering::Relaxed)
}
