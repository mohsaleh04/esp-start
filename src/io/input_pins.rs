use crate::io::{PinConfig, interrupt};
use esp_hal::gpio::{Input, InputPin};

pub const PRIMARY_BUTTON_DEBOUNCE_MS: u64 = 25;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonId {
    Primary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputEvent {
    ButtonPressed(ButtonId),
    ButtonReleased(ButtonId),
}

pub fn setup_primary_button(pin: impl InputPin + 'static) {
    interrupt::init_primary_button(Input::new(pin, PinConfig::PullUp.as_input()))
}

pub fn next_input_event() -> Option<InputEvent> {
    interrupt::next_event()
}

pub fn primary_button_is_pressed() -> bool {
    interrupt::primary_button_is_pressed()
}
