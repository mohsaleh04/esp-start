use crate::io::{ButtonId, InputEvent};
use esp_hal::gpio::Input;

mod debounce;
pub(super) mod handler;
mod states;

pub(super) fn init_button(button_id: ButtonId, button: Input<'static>) {
    states::init_button(button_id, button)
}

pub(super) fn button_is_pressed(button_id: ButtonId) -> bool {
    states::button_is_pressed(button_id)
}

pub(super) fn next_event() -> Option<InputEvent> {
    states::next_event()
}
