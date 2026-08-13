use crate::timer::scheduler::TIMER;
use crate::timer::states::TIMER_COUNTER;
use core::sync::atomic::Ordering;
use esp_hal::handler;

#[handler]
pub fn timer_handler() {
    critical_section::with(|cs| {
        let mut timer = TIMER.borrow_ref_mut(cs);
        if let Some(timer) = timer.as_mut() {
            timer.clear_interrupt();
        }
    });

    TIMER_COUNTER.fetch_add(1, Ordering::Relaxed);
}
