#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::spi::SpiBus;

use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    main,
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::Rate,
};

use esp_start::utils::delay;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

// Don't remove this
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config =
        esp_hal::Config::default()
            .with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(config);

    // ---------------------------------------------------------
    // GPIO
    // ---------------------------------------------------------

    // Backlight
    let mut backlight = Output::new(
        peripherals.GPIO22,
        Level::High,
        OutputConfig::default(),
    );

    // Reset: active LOW
    // We intentionally create it LOW so the display starts
    // in reset while the other pins are being configured.
    let mut rst = Output::new(
        peripherals.GPIO21,
        Level::High,
        OutputConfig::default(),
    );

    // D/C:
    // LOW  = command
    // HIGH = display data
    let mut dc = Output::new(
        peripherals.GPIO19,
        Level::Low,
        OutputConfig::default(),
    );

    // SCE / CS is active LOW
    let mut cs = Output::new(
        peripherals.GPIO5,
        Level::High,
        OutputConfig::default(),
    );

    // ---------------------------------------------------------
    // SPI3 / VSPI
    //
    // SCLK -> GPIO18
    // MOSI -> GPIO23
    // ---------------------------------------------------------

    let mut spi = Spi::new(
        peripherals.SPI3,
        SpiConfig::default()
            // Deliberately very slow for initial debugging.
            .with_frequency(Rate::from_mhz(1))
            .with_mode(Mode::_0),
    )
        .expect("failed to configure SPI")
        .with_sck(peripherals.GPIO18)
        .with_mosi(peripherals.GPIO23);

    // Give power / GPIOs some time to settle.
    delay(20);

    // ---------------------------------------------------------
    // RESET
    // ---------------------------------------------------------

    cs.set_high();

    rst.set_low();
    delay(10);

    rst.set_high();
    delay(10);

    // ---------------------------------------------------------
    // COMMAND 0x21
    //
    // Function set:
    // PD = 0 -> active
    // V  = 0 -> horizontal addressing
    // H  = 1 -> extended instruction set
    // ---------------------------------------------------------

    dc.set_low();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&[0x21]).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    delay(1);

    // ---------------------------------------------------------
    // COMMAND 0x04
    //
    // Temperature coefficient = 0
    // Same default used by the Linux driver.
    // ---------------------------------------------------------

    dc.set_low();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&[0x04]).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    delay(1);

    // ---------------------------------------------------------
    // COMMAND 0x14
    //
    // Bias = 4
    //
    // 0x10 | 4 = 0x14
    // Same value used by the Linux driver.
    // ---------------------------------------------------------

    dc.set_low();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&[0x14]).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    delay(1);

    // ---------------------------------------------------------
    // COMMAND 0xA8
    //
    // VOP / contrast = 40
    //
    // 0x80 | 40 = 0xA8
    //
    // Linux driver:
    // DEFAULT_GAMMA "40"
    // ---------------------------------------------------------

    dc.set_low();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&[0xB8]).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    delay(1);

    // ---------------------------------------------------------
    // COMMAND 0x22
    //
    // Function set:
    // PD = 0 -> active
    // V  = 1 -> vertical addressing
    // H  = 0 -> basic instruction set
    //
    // Exactly like the Linux driver.
    // ---------------------------------------------------------

    dc.set_low();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&[0x22]).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    delay(1);

    // ---------------------------------------------------------
    // COMMAND 0x0C
    //
    // Normal display mode
    // ---------------------------------------------------------

    dc.set_low();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&[0x0C]).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    delay(1);

    // ---------------------------------------------------------
    // SET X = 0
    // ---------------------------------------------------------

    dc.set_low();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&[0x80]).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    delay(1);

    // ---------------------------------------------------------
    // SET Y = 0
    // ---------------------------------------------------------

    dc.set_low();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&[0x40]).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    delay(1);

    // ---------------------------------------------------------
    // DISPLAY DATA
    //
    // 84 * 48 / 8 = 504 bytes
    //
    // Every bit = 1 => every pixel ON
    // ---------------------------------------------------------

    let framebuffer = [0xA0u8; 504];

    dc.set_high();
    delay(1);

    cs.set_low();
    delay(1);

    spi.write(&framebuffer).unwrap();
    spi.flush().unwrap();

    cs.set_high();
    //
    // // ---------------------------------------------------------
    // // Finished
    // // ---------------------------------------------------------
    //
    // loop {
    //     // Blink backlight just so we know firmware is alive.
    //     backlight.toggle();
    //     delay(1000);
    // }
    loop {}
}
