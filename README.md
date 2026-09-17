# esp-start

`esp-start` is a long-running educational embedded Rust project for the classic dual-core ESP32. Its goal is to grow from hardware bring-up experiments into a small, self-contained embedded computer with a display, input, removable storage, networking, a user interface, a shell, and a file manager.

The firmware is `no_std`. It deliberately keeps lower-level mechanisms such as shared SPI, framebuffer graphics, drawing algorithms, interrupts, storage, and networking visible instead of hiding them behind a large framework.

## Current capabilities

- Hand-written Nokia 5110 / PCD8544 driver with a 504-byte framebuffer
- Pixel, Bresenham line, rectangle, rounded rectangle, circle, and ASCII text rendering
- Full-frame LCD flushing with drawing kept separate from hardware I/O
- FAT-formatted microSD access on the SPI bus shared with the LCD
- Two interrupt-driven, active-low buttons with non-blocking debounce and an event queue
- Digital LED blinking and complementary PWM fading modes
- Periodic timer and pulse-counter experiments
- Optional Wi-Fi station connection, DHCP, and a retrying TCP/HTTP demonstration client
- An initial event loop suitable for evolving into a small OS-like application environment

See [DOC.md](DOC.md) for the architecture, implementation details, constraints, and planned direction.

## Target hardware

- Classic ESP32, dual core, 240 MHz, approximately 4 MB flash
- Nokia 5110 LCD using the PCD8544 controller, 84 x 48 monochrome pixels
- FAT-formatted microSD card
- CP2102 USB-to-UART bridge

## Wiring

| Device signal | ESP32 pin | Notes |
|---|---:|---|
| LCD backlight | GPIO22 | Digital output |
| LCD reset | GPIO21 | LCD-specific control |
| LCD data/command | GPIO17 | LCD-specific control |
| LCD chip select | GPIO5 | Independent SPI chip select |
| SPI SCK | GPIO18 | Shared by LCD and SD |
| SPI MOSI | GPIO23 | Shared by LCD and SD |
| SPI MISO | GPIO19 | Used by SD |
| SD chip select | GPIO16 | Independent SPI chip select |
| Backlight button | GPIO32 | Active-low, internal pull-up |
| LED-mode button | GPIO27 | Active-low, internal pull-up |
| Blink LED | GPIO25 | Digital output |
| PWM fade LED A | GPIO26 | First PWM output |
| PWM fade LED B | GPIO14 | Complementary PWM output |
| PCNT input | GPIO33 | Internal pull-up |
| UART0 TX | GPIO1 | Serial logging |
| UART0 RX | GPIO3 | Serial input |

`src/bin/main.rs` is the authoritative wiring reference. The LCD and SD card share SPI2 but have independent chip-select lines. The shared-bus design is intentional and must be preserved when extending either driver.

## Wi-Fi configuration

Wi-Fi credentials are read at compile time and must never be committed to the repository. Set them only in the local build environment:

```bash
ESP_START_WIFI_SSID='your-ssid' \
ESP_START_WIFI_PASSWORD='your-password' \
cargo build
```

`ESP_START_WIFI_PASSWORD` is optional at the source level to support open networks. If `ESP_START_WIFI_SSID` is absent, networking is skipped and the firmware continues into the main event loop in offline mode.

The demonstration HTTP endpoint is currently configured as constants in `src/bin/main.rs`. The request uses `Connection: close`, and the client reconnects and sends the same request again after five seconds. Do not point it at a non-idempotent endpoint without first changing the retry policy.

## Toolchain

The project uses the ESP Rust ecosystem and targets `xtensa-esp32-none-elf`. The repository's `Cargo.toml`, `Cargo.lock`, and CI configuration are authoritative because ESP HAL APIs evolve quickly.

Install and activate a compatible ESP Rust toolchain with `espup`. Before running commands that link the firmware, source the export script produced by `espup` so `xtensa-esp32-elf-gcc` is available.

## Build and quality checks

Run the complete local quality gate:

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```

Build-time Wi-Fi variables may be supplied to these commands when networking is required. Missing credentials do not prevent compilation.

## Flash and monitor

With `espflash` installed and the board connected:

```bash
espflash flash --monitor target/xtensa-esp32-none-elf/debug/esp-start
```

UART0 is the primary diagnostic channel. It reports hardware initialization, SD contents, input events, pulse-counter changes, Wi-Fi/DHCP status, and TCP activity.

## Project direction

The next major milestone is a primitive file manager that combines the existing display, input, and storage subsystems. This is not intended to become a Linux-like kernel. The goal is a compact integrated environment with drivers, services, a UI or shell, and small applications while retaining the educational value of the lower-level implementation.
