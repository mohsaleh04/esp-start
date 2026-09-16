use core::cell::RefCell;
use critical_section::Mutex;
use esp_hal::Blocking;
use esp_hal::time::Duration;
use esp_hal::timer::PeriodicTimer;

pub(super) static TIMER: Mutex<RefCell<Option<PeriodicTimer<'static, Blocking>>>> =
    Mutex::new(RefCell::new(None));

pub(super) fn schedule_timer(timer: PeriodicTimer<'static, Blocking>, every_millis: u64) {
    critical_section::with(|cs| {
        TIMER.borrow_ref_mut(cs).replace(timer);
    });
    critical_section::with(|cs| {
        let mut timer = TIMER.borrow_ref_mut(cs);
        let timer = timer.as_mut().unwrap();

        timer.start(Duration::from_millis(every_millis)).unwrap();
        timer.listen();
    });
}
