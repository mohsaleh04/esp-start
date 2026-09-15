use crate::io::InputEvent;
use esp_hal::gpio::Input;

pub(super) mod handler;
mod states;

pub(super) fn primary_button_is_pressed() -> bool {
    states::primary_button_is_pressed()
}

pub(super) fn init_primary_button(button: Input<'static>) {
    states::init_primary_button(button)
}

pub(super) fn next_event() -> Option<InputEvent> {
    states::next_event()
}
