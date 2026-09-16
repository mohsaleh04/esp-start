# esp-start

`esp-start` یک پروژهٔ آموزشی و بلندمدت embedded Rust برای ESP32 کلاسیک است. هدف، ساختن پایهٔ یک کامپیوتر کوچک و یکپارچه است: نمایشگر، ورودی، حافظهٔ SD، شبکه و در ادامه UI، shell و file manager.

این پروژه `no_std` است و عمداً بخش‌های پایین‌سطحی مثل SPI مشترک، framebuffer، الگوریتم‌های رسم، interruptها و networking را شفاف نگه می‌دارد.

## قابلیت‌های فعلی

- درایور دستی Nokia 5110 / PCD8544 با framebuffer 504 بایتی
- رسم pixel، Bresenham line، shape و متن ASCII
- microSD و FAT32 روی SPI مشترک با LCD
- ورودی‌های interrupt-based با debounce
- LED، PWM، timer و PCNT
- اتصال Wi-Fi، DHCP و HTTP client آزمایشی
- event loop اولیه برای یک محیط mini OS-like

جزئیات معماری و تصمیم‌های سخت‌افزاری در [DOC.md](DOC.md) آمده‌اند.

## سخت‌افزار و سیم‌کشی

| Device signal | ESP32 pin |
|---|---|
| LCD BL | GPIO22 |
| LCD RST | GPIO21 |
| LCD DC | GPIO17 |
| LCD CS | GPIO5 |
| SPI SCK | GPIO18 |
| SPI MOSI | GPIO23 |
| SPI MISO | GPIO19 |
| SD CS | GPIO16 |
| Backlight button | GPIO32 (active-low, internal pull-up) |
| LED mode button | GPIO27 (active-low, internal pull-up) |
| Blink LED | GPIO25 |
| PWM fade LED A | GPIO26 |
| PWM fade LED B | GPIO14 |
| PCNT input | GPIO33 (internal pull-up) |

مرجع نهایی wiring همیشه `src/bin/main.rs` است. LCD و SD روی SPI2 مشترک‌اند و CS مستقل دارند؛ معماری bus sharing بخشی از پایداری فعلی پروژه است.

## تنظیم Wi-Fi

credentialها داخل source یا repository ذخیره نمی‌شوند. آن‌ها را فقط در محیط local هنگام build قرار دهید:

```bash
ESP_START_WIFI_SSID='your-ssid' ESP_START_WIFI_PASSWORD='your-password' cargo build
```

بدون `ESP_START_WIFI_SSID`، firmware شبکه را skip می‌کند و offline وارد event loop می‌شود.

## Build و بررسی کیفیت

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```

فلش و monitor:

```bash
espflash flash --monitor target/xtensa-esp32-none-elf/debug/esp-start
```
