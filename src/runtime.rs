use esp_hal::peripherals::FROM_CPU_INTR0;
use esp_hal::ram;
use esp_hal::timer::timg::Timer;

pub fn allocate_heap() {
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);
}

pub fn setup_scheduler(interrupt: FROM_CPU_INTR0<'static>, timer: Timer<'static>) {
    esp_rtos::start(timer, interrupt);
}
