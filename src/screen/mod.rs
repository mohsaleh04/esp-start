pub use crate::screen::commands::config::DEFAULT_CONTRAST;
pub use crate::screen::controller::ScreenController;
pub use crate::screen::driver::ScreenDriver;

mod commands;
mod controller;
mod drawer_shapes;
mod drawer_text;
mod driver;
mod fmt;
mod font;
mod framebuffer;
mod spi;

pub const SCREEN_WIDTH: usize = 84;
pub const SCREEN_HEIGHT: usize = 48;
pub const SCREEN_CENTER: (usize, usize) = (SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
