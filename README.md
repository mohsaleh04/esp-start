# I've decided to start working with my new ESP32 as a hobby in Rust.
###### minimal project in rust

A small playground for learning embedded Rust on the **ESP32-WROOM-32D**.
I'm using this project to experiment with ESP32 hardware and understand embedded programming at a lower level — without Arduino or C/C++.

Currently playing with:

* GPIO and LEDs
* UART communication
* ESP32 memory and peripherals
* Bare-metal Rust with `no_std`
* `esp-hal` and `espflash`

Nothing serious yet, just learning how things actually work under the hood. :)

## Hardware

* ESP32-DevKitC
* ESP32-WROOM-32D
* ESP32-D0WD-V3
* 4 MB Flash
* CP2102 USB-to-UART

## Build

```bash
cargo build
```

Flash and monitor:

```bash
espflash flash --monitor target/xtensa-esp32-none-elf/debug/esp-start
```

---

Built while learning Rust, embedded systems, and probably breaking a few LEDs along the way. 🦀
