use crate::io::interrupt::states;
use esp_hal::handler;

#[handler]
pub fn gpio_handler() {
    states::handle_gpio_interrupts();
}
