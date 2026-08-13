use crate::io::interrupt::states;
use crate::io::interrupt::states::TEST_BTN;
use esp_hal::handler;

fn test_btn_handler() {
    critical_section::with(|cs| {
        let mut btn = TEST_BTN.borrow_ref_mut(cs);
        if let Some(btn) = btn.as_mut() {
            btn.clear_interrupt();
            states::set_test_button_pressed(btn.is_low());
        }
    });
}

#[handler]
pub fn gpio_handler() {
    test_btn_handler()
}
