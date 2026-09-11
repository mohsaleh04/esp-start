use esp_hal::gpio::{Input, InputPin};
use crate::io::{interrupt, PinConfig};

pub fn setup_test_button(pin: impl InputPin + 'static) {
    interrupt::init_test_button(Input::new(pin, PinConfig::PullUp.as_input()))
}

pub fn test_button_pressed() -> bool {
    interrupt::is_test_button_pressed()
}
