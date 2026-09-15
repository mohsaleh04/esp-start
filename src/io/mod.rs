use crate::io::interrupt::handler::gpio_handler;
use esp_hal::gpio::Io;
use esp_hal::peripherals::IO_MUX;

mod input_pins;
mod interrupt;
mod output_pins;
mod pins_config;

pub use input_pins::{
    next_input_event, primary_button_is_pressed, setup_primary_button, ButtonId, InputEvent,
};
pub use output_pins::{ScreenOutPins, ScreenPins, SdOutPins};
pub use pins_config::PinConfig;

pub fn setup(io_mux: IO_MUX<'static>) {
    let mut io = Io::new(io_mux);
    io.set_interrupt_handler(gpio_handler);
}
