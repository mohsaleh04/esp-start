pub use crate::screen::controller::ScreenController;
pub use crate::screen::driver::ScreenDriver;
pub use crate::screen::commands::config::DEFAULT_CONTRAST;

mod commands;
mod driver;
mod controller;
mod drawer_shapes;
mod drawer_text;
mod font;
mod spi;
mod fmt;

pub const SCREEN_WIDTH: usize = 84;
pub const SCREEN_HEIGHT: usize = 48;
pub const SCREEN_CENTER: (usize, usize) = (SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);

pub(crate) fn validate_bounds(x: u8, y: u8) -> bool {
    x < SCREEN_WIDTH as u8 && y < SCREEN_HEIGHT as u8
}
