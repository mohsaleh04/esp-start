use crate::io::{PinConfig, interrupt};
use esp_hal::gpio::{Input, InputPin};

pub const BUTTON_DEBOUNCE_MS: u64 = 25;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonId {
    Backlight,
    LedMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputEvent {
    ButtonPressed(ButtonId),
    ButtonReleased(ButtonId),
}

pub fn setup_backlight_button(pin: impl InputPin + 'static) {
    setup_button(ButtonId::Backlight, pin);
}

pub fn setup_led_mode_button(pin: impl InputPin + 'static) {
    setup_button(ButtonId::LedMode, pin);
}

fn setup_button(button_id: ButtonId, pin: impl InputPin + 'static) {
    interrupt::init_button(button_id, Input::new(pin, PinConfig::PullUp.as_input()));
}

pub fn next_input_event() -> Option<InputEvent> {
    interrupt::next_event()
}

pub fn button_is_pressed(button_id: ButtonId) -> bool {
    interrupt::button_is_pressed(button_id)
}
