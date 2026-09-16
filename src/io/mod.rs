use crate::io::interrupt::handler::gpio_handler;
use esp_hal::gpio::Io;
use esp_hal::peripherals::IO_MUX;

mod input_pins;
mod interrupt;
mod output_pins;
mod pins_config;

pub use input_pins::{
    BUTTON_DEBOUNCE_MS, ButtonId, InputEvent, button_is_pressed, next_input_event,
    setup_backlight_button, setup_led_mode_button,
};
pub use output_pins::{ScreenOutPins, ScreenPins, SdOutPins};
pub use pins_config::PinConfig;

pub fn setup(io_mux: IO_MUX<'static>) {
    let mut io = Io::new(io_mux);
    io.set_interrupt_handler(gpio_handler);
}
