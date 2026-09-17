# esp-start Architecture and Development Guide

This document is the technical map of `esp-start`. It records what the firmware currently does, why important architectural choices were made, and which constraints should guide future work. It is intended to make the project understandable even after a long break from development.

## 1. Project purpose

`esp-start` is a `no_std` embedded Rust project for the classic ESP32. Its long-term goal is a small, integrated embedded computer rather than a collection of unrelated hardware demonstrations.

The intended architecture is:

```text
boot and runtime
        |
hardware abstractions and drivers
        |
display + input + storage + networking
        |
UI, shell, file manager, and small applications
```

The term "mini OS" describes this integration. It does not mean implementing a Linux-like kernel. The project prioritizes learning and explicit ownership of the mechanisms involved: SPI, framebuffer layout, drawing algorithms, interrupts, asynchronous networking, storage, and constrained-memory design.

The source currently contains working implementations for:

- Nokia 5110 / PCD8544 display control
- A monochrome framebuffer with shape and ASCII text drawing
- Shared SPI access for the LCD and microSD card
- FAT volume mounting and root-directory traversal
- Two interrupt-driven buttons with debounce and queued events
- UART logging, PWM LEDs, a periodic hardware timer, and PCNT
- Wi-Fi station setup, DHCP, TCP, and a small HTTP demonstration client
- An Embassy-driven main event loop

## 2. Hardware and wiring

The final authority for pin assignments is `src/bin/main.rs`.

| Component or signal | ESP32 pin | Purpose |
|---|---:|---|
| LCD BL | GPIO22 | Backlight control |
| LCD RST | GPIO21 | Display reset |
| LCD DC | GPIO17 | Command/data selection |
| LCD CS | GPIO5 | LCD chip select |
| SPI SCK | GPIO18 | Shared SPI clock |
| SPI MOSI | GPIO23 | Shared controller-to-device data |
| SPI MISO | GPIO19 | SD-to-controller data |
| SD CS | GPIO16 | SD card chip select |
| Backlight button | GPIO32 | Active-low with internal pull-up |
| LED-mode button | GPIO27 | Active-low with internal pull-up |
| Blink LED | GPIO25 | Digital LED output |
| Fade LED A | GPIO26 | First PWM output |
| Fade LED B | GPIO14 | Complementary PWM output |
| PCNT input | GPIO33 | Pulse-counter input with internal pull-up |
| UART0 TX | GPIO1 | Serial output |
| UART0 RX | GPIO3 | Serial input |

Each button is connected between its GPIO and ground. The internal pull-up gives these states:

```text
released -> HIGH
pressed  -> LOW
```

GPIO32 toggles the LCD backlight. GPIO27 advances the LED state machine through `Off -> Blink -> Fade -> Off`.

The SPI clock is intentionally conservative at approximately 1 MHz. Hardware stability takes priority over increasing the bus speed without measurement.

## 3. Boot flow

The entry point is `src/bin/main.rs`, using `#[esp_rtos::main]` with an Embassy `Spawner`. The current boot sequence is:

1. Allocate the heap required by the radio stack.
2. Initialize the ESP32 at the maximum configured CPU clock.
3. Configure UART0 for boot and runtime diagnostics.
4. Install GPIO interrupt support and configure both buttons.
5. Configure the periodic timer, scheduler support, and LED controller.
6. Create the shared SPI2 bus.
7. Create an independent SPI device for the LCD, initialize the display, clear it, flush it, and turn on the backlight.
8. Create an independent SPI device for the SD card, mount the first FAT volume, display capacity information, and list the root directory.
9. If credentials exist, initialize Wi-Fi, connect, wait for DHCP, and create the HTTP client.
10. Configure PCNT unit 0 channel 0 on GPIO33.
11. Enter the main event loop.

LCD initialization or its initial flush is currently fatal because the display is a central output device. SD and network failures are non-fatal. Missing media, missing credentials, connection failures, and network timeouts are logged, after which the firmware continues into the event loop.

Boot is still sequential. Wi-Fi association and DHCP may delay entry to the main loop, but both waits are bounded.

## 4. Shared SPI architecture

The LCD and SD card share the SPI2 clock and data lines while retaining independent chip-select signals:

```text
ESP32 SPI2
 |-- SCK  GPIO18 ---- LCD + SD
 |-- MOSI GPIO23 ---- LCD + SD
 |-- MISO GPIO19 -------- SD
 |-- CS   GPIO5  -------- LCD
 `-- CS   GPIO16 -------- SD
```

`src/spi_bus.rs` constructs the bus. `embedded_hal_bus::spi::RefCellDevice` creates a device view for each peripheral and manages access to the shared bus. Each device transaction asserts only that device's chip select. LCD-specific DC and reset signals remain outside the generic SPI transaction mechanism.

This design is the result of real hardware bring-up work. Directly giving the bus to one peripheral caused failures when SD support was introduced. Do not replace the shared-device architecture with direct SPI ownership unless the interaction between bus locking, chip select, and both drivers is fully understood and verified on hardware.

## 5. Display and framebuffer

The PCD8544 display is 84 pixels wide, 48 pixels high, and monochrome. Its framebuffer occupies:

```text
84 * 48 / 8 = 504 bytes
```

The controller divides the vertical dimension into six banks, each eight pixels high:

```text
bank 0 -> y 0..7
bank 1 -> y 8..15
...
bank 5 -> y 40..47
```

Pixel addressing follows:

```text
bank  = y / 8
index = x + bank * 84
bit   = y % 8
```

Setting and clearing a pixel are integer bit operations:

```rust
buffer[index] |= 1 << bit;
buffer[index] &= !(1 << bit);
```

### Drawing and I/O separation

`Framebuffer` in `src/screen/framebuffer.rs` owns the 504-byte RAM image. Drawing operations modify this image only. They do not perform SPI transactions and do not return SPI errors.

The operations include pixel, line, rectangle, rounded rectangle, circle, filled shape, character, text, and formatted-text drawing. The image is transferred to the LCD only by:

```rust
screen.flush()?;
```

`flush()` currently sends the entire framebuffer. A 504-byte full-frame update is an acceptable simplicity and reliability tradeoff at this stage. Dirty-bank or dirty-region rendering should be introduced only after a measured need.

### Drawing algorithms

Lines use Bresenham's integer algorithm. An earlier educational DDA implementation was removed because floating-point slope calculations offer no advantage for this display. Circles and rounded corners use midpoint-style integer algorithms and symmetry. Filled shapes are constructed with integer spans or internal lines.

### Text rendering

The ASCII font is stored in `src/screen/font.rs` as columns of glyph bits. `draw_text` renders a string, while `draw_fmt` accepts `format_args!` and supports formatted output without allocating a heap `String`.

`ScreenFmtWriter` preserves the starting X coordinate as its line anchor. Text that starts at `(10, 4)` returns to X=10 after explicit newlines and soft wrapping, rather than returning to the left edge.

### Display module responsibilities

| File | Responsibility |
|---|---|
| `commands.rs` | PCD8544 command values |
| `driver.rs` | SPI and the DC, reset, and backlight pins |
| `framebuffer.rs` | The 504-byte image and pixel operations |
| `controller.rs` | Initialization, clear, flush, and backlight control |
| `drawer_shapes.rs` | Geometric drawing algorithms |
| `drawer_text.rs` | Character and string rendering |
| `fmt.rs` | `core::fmt::Write` integration |
| `font.rs` | ASCII glyph data |
| `spi.rs` | Display SPI helpers and related types |

Hardware initialization necessarily sends contrast, bias, instruction-set, addressing, and display-mode commands. After initialization, ordinary drawing remains a RAM-only operation until `flush()`.

## 6. SD storage

`SdStorage` in `src/sd/volume.rs` hides the concrete `SdCard`, `VolumeManager`, volume index, and open-volume lifetime from `main`.

A typical mount is:

```rust
let storage = SdStorage::new(sd_spi, delay);

match storage.mount() {
    Ok(sd) => {
        let bytes = sd.size_bytes();
    }
    Err(SdStorageError::CardNotFound) => {
        // Continue without removable storage.
    }
    Err(error) => {
        // Report a transport, partition, or filesystem error.
    }
}
```

`MountedSd` is an RAII handle. The FAT volume remains open only while this value is alive and closes when it is dropped. In the current boot demonstration, the handle lives inside the successful `storage.mount()` match arm, lists the root directory, and is then dropped. A file manager must move `MountedSd` into long-lived application state so the volume remains mounted across the event loop.

Storage errors are deliberately distinguished:

- `CardNotFound`: no card is present or the card was removed.
- `Card(...)`: an SD protocol or transport error occurred.
- `Filesystem(...)`: partition, FAT, or filesystem operations failed.

Errors opening or traversing the root directory are returned to the caller rather than silently ignored.

`FixedTimeSource` supplies a constant timestamp required by the FAT library. It is not a real clock; future file writes will have artificial timestamps until an RTC or network-derived time source is added.

## 7. Input, interrupts, and debounce

The public input API in `src/io/input_pins.rs` includes:

```rust
setup_backlight_button(pin);
setup_led_mode_button(pin);
next_input_event();
button_is_pressed(ButtonId::Backlight);
```

The event model supports press and release transitions for both buttons:

```rust
InputEvent::ButtonPressed(ButtonId::Backlight)
InputEvent::ButtonPressed(ButtonId::LedMode)
InputEvent::ButtonReleased(ButtonId::Backlight)
InputEvent::ButtonReleased(ButtonId::LedMode)
```

Events are used instead of polling only the current Boolean level so short transitions are not lost between iterations of the main loop.

Mechanical contacts can produce several rapid edges for a single physical transition. The current debounce interval is 25 ms and remains non-blocking:

1. An any-edge GPIO interrupt runs.
2. The handler clears the interrupt flag.
3. It records the latest raw level and edge time.
4. No delay or heavy work is performed inside the ISR.
5. `next_input_event()` advances pending debounce state.
6. After 25 ms without a newer edge, the stable level is accepted.
7. An event is queued only when the accepted state differs from the previous stable state.

Each button has independent state. Accepted events enter a fixed ring buffer with capacity eight, protected by `critical_section` between interrupt and main contexts. The queue uses no heap. If it is full, a new event is dropped without telemetry; overflow reporting is a useful future improvement.

## 8. Main event loop

The event loop coordinates all active behavior:

```rust
loop {
    while let Some(event) = io::next_input_event() {
        // Toggle the backlight, change LED mode, or log a release.
    }

    leds.update();
    // Log PCNT changes.
    // Poll the HTTP client when networking is available.
    Timer::after(Duration::from_millis(1)).await;
}
```

The one-millisecond Embassy timer yields execution so the network runner can progress even when no TCP traffic is arriving.

The current event paths are:

```text
GPIO32 -> interrupt -> debounce -> event queue -> LCD backlight
GPIO27 -> interrupt -> debounce -> event queue -> LED state machine
```

As the UI grows, the same event stream can be routed to a menu, file manager, or active application rather than handled directly in `main`.

## 9. LEDs, PWM, timer, and pulse counter

`src/pwm/` configures the ESP32 LEDC timer and channels. `src/leds.rs` adds the higher-level `Off`, `Blink`, and `Fade` state machine, including fade direction and update timing.

`PwmController::set_duty` only changes a duty cycle and intentionally contains no delay. Callers own timing and scheduling:

```rust
pwm.set_duty(50);
// The caller or scheduler determines how long this state lasts.
pwm.off();
```

This prevents a hardware setter from unexpectedly blocking the CPU or event loop. GPIO26 and GPIO14 form the complementary fade pair; GPIO25 is the digital blink output.

`src/timer/` configures a periodic hardware timer. Its interrupt handler clears the interrupt and increments an atomic counter. It is a focused experiment in periodic interrupts and atomic communication, not a general-purpose scheduler abstraction.

PCNT unit 0 channel 0 counts configured edges on GPIO33. The main loop compares the current value with the last reported value and logs changes over UART.

## 10. UART and error reporting

UART0 uses GPIO1 and GPIO3. It is the primary diagnostic channel for initialization results, SD directory entries, button events, PCNT values, and network activity.

The LCD is reserved for concise user-facing status. Detailed errors normally go to UART. The panic handler currently spins indefinitely and does not print panic information, so explicit error logging before fatal loops remains important.

## 11. Wi-Fi and networking

Credentials are read at compile time:

```bash
ESP_START_WIFI_SSID='your-ssid' \
ESP_START_WIFI_PASSWORD='your-password' \
cargo build
```

They must not be stored in source control. If the SSID is absent, network setup is skipped. Password is optional for an open network.

The network path:

1. Initializes the ESP radio and station interface.
2. Applies the station configuration.
3. Creates an `embassy-net` stack using DHCPv4.
4. Spawns the network runner task.
5. Associates with the access point.
6. Waits for a usable IPv4 configuration.
7. Creates one TCP socket, connects to the configured endpoint, and sends the static request.
8. Polls response chunks from the main event loop.

Failure remains non-fatal and all long waits are bounded:

| Operation | Timeout or interval |
|---|---:|
| Wi-Fi association | 20 seconds |
| DHCP configuration | 15 seconds |
| TCP connection | 10 seconds |
| TCP read polling | 10 milliseconds maximum per poll |
| Reconnect after failure or closure | 5 seconds |

`HttpClient` retains the endpoint and a `&'static [u8]` request so it can reconnect and resend. The current request asks for `Connection: close`, so an ordinary server close schedules that request again after five seconds. This policy is safe only for idempotent endpoints.

RX and TX storage are singleton `StaticCell<[u8; 1024]>` buffers. Consequently, the present architecture supports exactly one `TcpSocket`, even though the network stack itself has multiple socket resources. Supporting multiple network applications requires a buffer pool or explicit independent buffer ownership.

The response buffer is 512 bytes. Chunks are logged only when they are valid UTF-8. There is no complete HTTP parser, response assembly, TLS, or general service layer yet.

## 12. Source map

| Path | Responsibility |
|---|---|
| `src/bin/main.rs` | Boot, subsystem integration, and the main loop |
| `src/runtime.rs` | Heap allocation and scheduler/runtime setup |
| `src/spi_bus.rs` | Shared SPI2 construction |
| `src/screen/` | Display driver, framebuffer, graphics, and text |
| `src/sd/` | SD card access, FAT volume, and time source |
| `src/io/` | Pin configuration, GPIO interrupts, debounce, and input events |
| `src/leds.rs` | LED mode state machine |
| `src/pwm/` | LEDC PWM setup and control |
| `src/timer/` | Periodic timer and counter state |
| `src/pcnt/` | Pulse-counter channel configuration |
| `src/wifi/` | Radio initialization, station configuration, scanning, and association |
| `src/net/` | Embassy network stack, runner, TCP socket, and HTTP client |
| `src/com/uart.rs` | UART0 construction |
| `src/utils.rs` | Small utility functions, including network seed generation |
| `src/lib.rs` | Library module exports |

## 13. Toolchain, build, and execution

The project targets the Xtensa-based classic ESP32 using the ESP Rust toolchain. Dependency versions in `Cargo.toml` and `Cargo.lock` are authoritative; examples for older `esp-hal`, `esp-radio`, Embassy, or `embedded-hal` releases may not match the installed APIs.

Activate the toolchain environment generated by `espup` before linking. A failure that only reports a missing `xtensa-esp32-elf-gcc` usually means the export script was not sourced.

Run the full quality gate:

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```

Flash and monitor a debug build with:

```bash
espflash flash --monitor target/xtensa-esp32-none-elf/debug/esp-start
```

JetBrains `.idea/` files are machine-specific and intentionally excluded from version control.

## 14. Development principles

1. Preserve correctness and known hardware stability before optimizing.
2. Treat the shared SPI bus and independent chip-select behavior as architectural constraints.
3. Keep framebuffer drawing separate from hardware image transfer.
4. Keep ISRs short: no blocking delays, allocation, or heavy logging.
5. Distinguish missing removable media from transport and filesystem failures.
6. Keep timing in callers or schedulers rather than inside hardware setters.
7. Prefer bounded asynchronous waits over infinite boot-time waits.
8. Prefer fixed-size buffers with explicit ownership.
9. Prefer integer embedded algorithms such as Bresenham and midpoint drawing over floating point.
10. Verify dependency versions and installed APIs before applying examples from the internet.
11. Measure before adding dirty rendering, more complex scheduling, or other optimizations.
12. Keep `README.md`, this document, and actual source behavior synchronized.

## 15. Current limitations and next milestones

The following limitations are intentional or not yet addressed:

- Every LCD flush transfers all 504 framebuffer bytes.
- The input queue holds eight events and exposes no overflow counter.
- Two dedicated buttons exist, but there is no general navigation input model.
- FAT timestamps come from a fixed artificial time source.
- `MountedSd` is dropped after the boot-time root listing.
- There is no UI manager or application-level event router.
- A final keyboard or richer input device has not been selected.
- Networking uses one TCP socket with singleton buffers.
- The HTTP client has no complete parser, response assembly, or TLS.
- The server endpoint and HTTP request are compile-time constants in `main`.
- The main loop has no explicit power-management strategy.

The next coherent milestone is a primitive file manager. It should:

1. Move `MountedSd` into long-lived application state.
2. Define navigation actions such as Up, Down, Select, and Back.
3. Add a small menu or screen model.
4. Route input events outside `main` as the number of screens grows.
5. Browse directory entries and display them using the existing text renderer.
6. Add input-queue overflow telemetry.

Dirty-bank display updates, keyboard hardware, USB host support, TLS, multi-socket networking, and a full UI framework should remain outside that milestone unless a concrete requirement makes them necessary.
