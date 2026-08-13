use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use critical_section::Mutex;
use esp_hal::gpio::{Event, Input};

pub(super) static TEST_BTN: Mutex<RefCell<Option<Input<'static>>>> = Mutex::new(RefCell::new(None));
pub(super) static TEST_BTN_PRESSED: AtomicBool = AtomicBool::new(false);

pub(super) fn is_test_button_pressed() -> bool {
    TEST_BTN_PRESSED.load(Ordering::Relaxed)
}

pub(super) fn set_test_button_pressed(value: bool) {
    TEST_BTN_PRESSED.store(value, Ordering::Relaxed);
}

pub(super) fn init_test_button(test_btn: Input<'static>) {
    critical_section::with(|cs| {
        TEST_BTN.borrow_ref_mut(cs).replace(test_btn);
    });
    critical_section::with(|cs| {
        let mut this_btn = TEST_BTN.borrow_ref_mut(cs);
        let btn = this_btn.as_mut().unwrap();

        btn.listen(Event::AnyEdge);
    });
}
