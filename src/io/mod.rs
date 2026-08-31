use crate::io::interrupt::handler::gpio_handler;
use esp_hal::gpio::{Input, InputPin, Io};
use esp_hal::peripherals::IO_MUX;

mod interrupt;
mod output_pins;
mod pins_config;

pub use output_pins::OutputPins;
pub use pins_config::PinConfig;

pub fn setup(io_mux: IO_MUX<'static>, test_button: impl InputPin + 'static) {
    let mut io = Io::new(io_mux);
    io.set_interrupt_handler(gpio_handler);

    let test_button = Input::new(test_button, PinConfig::PullUp.as_input());
    interrupt::init_test_button(test_button)
}

pub fn test_button_pressed() -> bool {
    interrupt::is_test_button_pressed()
}
