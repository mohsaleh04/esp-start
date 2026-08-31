use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::peripherals::SW_INTERRUPT;
use esp_hal::ram;
use esp_hal::timer::timg::Timer;

pub fn allocate_heap() {
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);
}

pub fn setup_scheduler(sw_interrupt: SW_INTERRUPT<'static>, timer: Timer<'static>) {
    let sw_interrupt = SoftwareInterruptControl::new(sw_interrupt);
    esp_rtos::start(timer, sw_interrupt.software_interrupt0);
}
