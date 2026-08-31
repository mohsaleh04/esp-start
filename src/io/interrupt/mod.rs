use esp_hal::gpio::Input;

pub(super) mod handler;
mod states;

pub(super) fn is_test_button_pressed() -> bool {
    states::is_test_button_pressed()
}

pub(super) fn init_test_button(test_btn: Input<'static>) {
    states::init_test_button(test_btn)
}
