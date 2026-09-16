use crate::io::interrupt::states;
use crate::io::interrupt::states::PRIMARY_BUTTON;
use esp_hal::handler;

fn primary_button_handler() {
    critical_section::with(|cs| {
        let mut button = PRIMARY_BUTTON.borrow_ref_mut(cs);
        if let Some(button) = button.as_mut() {
            button.clear_interrupt();
            states::record_primary_button_edge(cs, button.is_low());
        }
    });
}

#[handler]
pub fn gpio_handler() {
    primary_button_handler()
}
