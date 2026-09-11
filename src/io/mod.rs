use crate::io::interrupt::handler::gpio_handler;
use esp_hal::gpio::Io;
use esp_hal::peripherals::IO_MUX;

mod input_pins;
mod output_pins;
mod pins_config;
mod interrupt;

pub use output_pins::ScreenOutPins;
pub use output_pins::ScreenPins;
pub use output_pins::SdOutPins;
pub use pins_config::PinConfig;

pub fn setup(io_mux: IO_MUX<'static>) {
    let mut io = Io::new(io_mux);
    io.set_interrupt_handler(gpio_handler);
}
