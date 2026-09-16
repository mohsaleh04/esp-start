# راهنمای پروژهٔ esp-start

این فایل یک نقشهٔ آموزشی از وضعیت فعلی پروژه است. هدفش این است که چند ماه بعد هم بتوانیم سریع بفهمیم هر قسمت چه کاری انجام می‌دهد، چرا به این شکل طراحی شده و برای ادامهٔ کار باید از کجا شروع کنیم.

## ۱. این پروژه چیست؟

`esp-start` یک پروژهٔ Rust از نوع `no_std` برای ESP32 کلاسیک است. هدف نهایی، ساختن یک محیط کوچک و یکپارچه شبیه یک کامپیوتر embedded است؛ نه یک سیستم‌عامل کامل مثل Linux.

تصویر کلی معماری مورد نظر:

```text
راه‌اندازی و main loop
        ↓
درایورها و abstractionهای سخت‌افزار
        ↓
نمایشگر + ورودی + حافظه + ارتباطات
        ↓
UI، فایل‌منیجر و برنامه‌های کوچک
```

در وضعیت فعلی این قسمت‌ها واقعاً در source وجود دارند:

- نمایشگر Nokia 5110 با کنترلر PCD8544
- framebuffer و رسم شکل و متن
- microSD و FAT volume
- SPI مشترک میان LCD و SD
- یک دکمهٔ interrupt-based با debounce
- UART برای log
- abstraction اولیهٔ PWM و timer
- اتصال Wi-Fi، DHCP و یک HTTP client آزمایشی

ماژول‌های فعال `src/wifi/` و `src/net/` اتصال station، پیکربندی DHCP، TCP و polling پاسخ HTTP را پیاده‌سازی می‌کنند.

## ۲. سخت‌افزار و سیم‌کشی فعلی

مرجع نهایی سیم‌کشی همیشه `src/bin/main.rs` است.

| قطعه یا سیگنال | پایهٔ ESP32 | توضیح |
|---|------------:|---|
| LCD BL |      GPIO22 | نور پس‌زمینه |
| LCD RST |      GPIO21 | reset نمایشگر |
| LCD DC |      GPIO17 | انتخاب command یا data |
| LCD CS |       GPIO5 | chip select نمایشگر |
| SPI SCK |      GPIO18 | کلاک مشترک LCD و SD |
| SPI MOSI |      GPIO23 | داده از ESP32 به دستگاه‌ها |
| SPI MISO |      GPIO19 | داده از SD به ESP32 |
| SD CS |      GPIO16 | chip select کارت SD |
| دکمهٔ backlight |      GPIO32 | active-low با pull-up داخلی |
| دکمهٔ حالت LED |      GPIO27 | active-low با pull-up داخلی |
| LED چشمک‌زن |      GPIO25 | خروجی دیجیتال |
| LED محوشونده A |      GPIO26 | کانال PWM اول |
| LED محوشونده B |      GPIO14 | کانال PWM دوم، مکمل کانال اول |
| ورودی شمارندهٔ پالس |      GPIO33 | PCNT با pull-up داخلی |
| UART TX |       GPIO1 | خروجی serial |
| UART RX |       GPIO3 | ورودی serial |

### اتصال دکمه

هر دکمه یک سمتش به GPIO خودش و سمت دیگرش به GND وصل می‌شود. چون pull-up داخلی فعال است:

```text
دکمه رها است  → GPIO = HIGH
دکمه فشرده است → GPIO = LOW
```

GPIO32 نور پس‌زمینهٔ LCD را toggle می‌کند. GPIO27 حالت LEDها را در چرخهٔ
`Off → Blink → Fade → Off` جلو می‌برد. برای این ورودی‌ها مقاومت pull-up خارجی لازم نیست،
هرچند در طراحی نهایی سخت‌افزار می‌توان مقاومت خارجی مناسب اضافه کرد.

## ۳. جریان راه‌اندازی برنامه

نقطهٔ ورود برنامه `src/bin/main.rs` است. ترتیب کلی boot:

1. ESP32 با بیشترین کلاک CPU راه‌اندازی می‌شود.
2. UART ساخته می‌شود تا مراحل boot قابل مشاهده باشند.
3. handler عمومی GPIO نصب و دکمه‌های GPIO32 و GPIO27 آماده می‌شوند.
4. timer دوره‌ای و controller مربوط به LEDها ساخته می‌شوند.
5. bus مشترک SPI2 ساخته می‌شود.
6. LCD و سپس SD با device و chip select مستقل راه‌اندازی می‌شوند.
7. framebuffer با `flush()` روی LCD فرستاده می‌شود.
8. Wi-Fi، DHCP و client شبکه راه‌اندازی می‌شوند.
9. PCNT روی GPIO33 آماده می‌شود.
10. برنامه وارد event loop می‌شود و ورودی، LED، PCNT و شبکه را پیش می‌برد.

نبودن SD یک خطای fatal نیست. سیستم پیام مناسب نشان می‌دهد و همچنان وارد event loop می‌شود. شبکه نیز optional است: نبودن تنظیمات، شکست اتصال Wi-Fi یا timeout در DHCP مانع ورود سیستم به event loop نمی‌شود.

## ۴. چرا LCD و SD یک SPI مشترک دارند؟

SPI معمولاً این خطوط را دارد:

- SCK: کلاک
- MOSI: داده از master به slave
- MISO: داده از slave به master
- CS: انتخاب دستگاه

LCD و SD سه خط اصلی SPI را شریک هستند، اما هرکدام CS جدا دارند:

```text
ESP32 SPI2
 ├── SCK  GPIO18 ── LCD + SD
 ├── MOSI GPIO23 ── LCD + SD
 ├── MISO GPIO19 ── SD
 ├── CS   GPIO5  ── LCD
 └── CS   GPIO16 ── SD
```

ماژول `src/spi_bus.rs` خود bus را می‌سازد. سپس `RefCellDevice` برای هر دستگاه یک نمای مستقل می‌سازد.

مزیت این روش:

- مالکیت مستقیم SPI به یک دستگاه داده نمی‌شود.
- هنگام transaction فقط CS همان دستگاه فعال می‌شود.
- LCD و SD بدون خراب‌کردن ارتباط یکدیگر از bus استفاده می‌کنند.

نباید `RefCellDevice` را بدون دلیل با مالکیت مستقیم SPI جایگزین کرد؛ خرابی LCD هنگام اضافه‌شدن SD قبلاً یکی از مشکلات واقعی پروژه بوده است.

## ۵. نمایشگر و framebuffer

نمایشگر PCD8544 ابعاد ۸۴×۴۸ دارد و تک‌رنگ است. حافظهٔ لازم:

```text
84 × 48 ÷ 8 = 504 bytes
```

کنترلر نمایشگر ارتفاع را به ۶ bank هشت‌پیکسلی تقسیم می‌کند:

```text
bank 0 → y = 0..7
bank 1 → y = 8..15
...
bank 5 → y = 40..47
```

فرمول دسترسی به pixel:

```text
bank  = y / 8
index = x + bank * 84
bit   = y % 8
```

برای روشن‌کردن pixel:

```rust
buffer[index] |= 1 << bit;
```

برای خاموش‌کردن آن:

```rust
buffer[index] &= !(1 << bit);
```

### جداسازی رسم از I/O

`Framebuffer` در `src/screen/framebuffer.rs` فقط دادهٔ RAM را نگه می‌دارد.

توابع زیر فقط framebuffer را تغییر می‌دهند:

- `draw_px`
- `draw_line`
- `draw_rect`
- `draw_round_rect`
- `draw_circle`
- `draw_char`
- `draw_text`
- `draw_fmt`

آن‌ها transaction مربوط به SPI انجام نمی‌دهند و خطای SPI برنمی‌گردانند.

تنها زمانی که تصویر واقعاً به LCD منتقل می‌شود، فراخوانی زیر است:

```rust
screen.flush()?;
```

در حال حاضر `flush()` همیشه هر ۵۰۴ بایت را ارسال می‌کند. dirty-region یا dirty-bank هنوز پیاده‌سازی نشده است؛ این تصمیم فعلاً عمدی است تا رفتار ساده و قابل‌اعتماد بماند.

### الگوریتم‌های رسم

خط‌ها با الگوریتم Bresenham رسم می‌شوند. این الگوریتم فقط از اعداد صحیح استفاده می‌کند و برای نمایشگر embedded مناسب است. پیاده‌سازی اولیهٔ DDA برای مقایسهٔ آموزشی نوشته شده بود، اما چون به محاسبات floating-point نیاز داشت از مسیر اجرایی و سورس حذف شد؛ این تاریخچه دلیل انتخاب Bresenham را توضیح می‌دهد.

دایره و گوشه‌های گرد با الگوریتم midpoint و تقارن چندجهته ساخته می‌شوند. filled shapeها نیز با رسم خط‌های داخلی پر می‌شوند.

### متن و wrapping

فونت ASCII در `src/screen/font.rs` نگهداری می‌شود. هر glyph مجموعه‌ای از ستون‌های بیتی است.

`draw_text` رشتهٔ معمولی می‌گیرد. `draw_fmt` اجازه می‌دهد از `format_args!` استفاده کنیم و بدون ساختن `String` روی heap متن قالب‌بندی‌شده رسم کنیم.

`ScreenFmtWriter` مختصات X اولیه را نگه می‌دارد. بنابراین اگر متن از `(10, 4)` شروع شود، newline و soft wrap نیز به X برابر ۱۰ برمی‌گردند، نه لبهٔ صفر نمایشگر.

## ۶. ساختار درایور نمایشگر

مسئولیت‌ها به این صورت جدا شده‌اند:

- `commands.rs`: commandهای PCD8544
- `driver.rs`: SPI، پایه‌های DC/RST و backlight
- `framebuffer.rs`: آرایهٔ ۵۰۴ بایتی و عملیات pixel
- `controller.rs`: initialization، clear، flush و کنترل backlight
- `drawer_shapes.rs`: شکل‌های هندسی
- `drawer_text.rs`: character و text
- `fmt.rs`: اتصال `core::fmt::Write` به renderer
- `font.rs`: دادهٔ فونت ASCII

`init()` ناچار است commandهای سخت‌افزاری تنظیم contrast، bias و addressing را ارسال کند. بعد از initialization، مسیرهای رسم فقط RAM را تغییر می‌دهند و I/O تصویر در `flush()` انجام می‌شود.

## ۷. حافظهٔ SD

`SdStorage` در `src/sd/volume.rs` جزئیات این typeها را از `main` پنهان می‌کند:

- `SdCard`
- `VolumeManager`
- `VolumeIdx`
- lifetime مربوط به volume باز

استفادهٔ اصلی:

```rust
let storage = SdStorage::new(sd_spi, delay);

match storage.mount() {
    Ok(sd) => {
        let bytes = sd.size_bytes();
    }
    Err(SdStorageError::CardNotFound) => {
        // کارت داخل دستگاه نیست
    }
    Err(error) => {
        // خطای واقعی card یا filesystem
    }
}
```

`MountedSd` یک handle از نوع RAII است. تا وقتی این مقدار زنده است volume باز می‌ماند و هنگام drop شدن بسته می‌شود.

در boot فعلی این handle فقط داخل شاخهٔ موفق `match storage.mount()` زنده می‌ماند، root را فهرست می‌کند و سپس drop می‌شود. این رفتار برای demo فعلی درست است؛ file manager باید مالکیت `MountedSd` را به state طولانی‌عمر برنامه منتقل کند تا volume در سراسر event loop mount بماند.

### خطاهای SD

خطاها سه مسیر واضح دارند:

- `CardNotFound`: کارت وجود ندارد یا هنگام کار جدا شده است.
- `Card(...)`: خطای protocol/transport کارت.
- `Filesystem(...)`: مشکل partition، FAT یا عملیات فایل‌سیستم.

خطای بازکردن root و پیمایش directory نیز به caller برگردانده می‌شود و silently ناپدید نمی‌شود.

### زمان جعلی فایل‌سیستم

`FixedTimeSource` فعلاً همیشه یک timestamp ثابت برمی‌گرداند. این برای API کتابخانهٔ FAT لازم است، ولی ساعت واقعی نیست.

در نتیجه timestamp فایل‌هایی که در آینده نوشته شوند واقعی نخواهد بود. تغییر این رفتار به RTC یا زمان شبکه یک کار آینده است.

## ۸. ورودی، interrupt و debounce

API عمومی ورودی در `src/io/input_pins.rs` قرار دارد:

```rust
setup_backlight_button(pin);
setup_led_mode_button(pin);
next_input_event();
button_is_pressed(ButtonId::Backlight);
```

رویدادهای فعلی:

```rust
InputEvent::ButtonPressed(ButtonId::Backlight)
InputEvent::ButtonPressed(ButtonId::LedMode)
InputEvent::ButtonReleased(ButtonId::Backlight)
InputEvent::ButtonReleased(ButtonId::LedMode)
```

### چرا فقط خواندن boolean کافی نبود؟

اگر برنامه فقط وضعیت فعلی را بخواند، ممکن است یک فشار کوتاه را از دست بدهد. event به برنامه می‌گوید یک transition اتفاق افتاده است، حتی اگر بعداً وضعیت تغییر کند.

### bounce چیست؟

کنتاکت مکانیکی دکمه هنگام فشار یا رهاشدن فوراً پایدار نمی‌شود و ممکن است چند بار تغییر کند:

```text
LOW → HIGH → LOW → HIGH → LOW
```

بدون debounce، یک فشار می‌تواند چند Pressed و Released تولید کند.

### debounce فعلی چگونه کار می‌کند؟

مدت debounce برابر ۲۵ms است.

1. interrupt با هر edge اجرا می‌شود.
2. handler فقط flag وقفه را پاک می‌کند.
3. آخرین سطح خام و زمان edge ثبت می‌شود.
4. هیچ delay داخل ISR انجام نمی‌شود.
5. `next_input_event()` وضعیت pending را poll می‌کند.
6. اگر ۲۵ms از آخرین edge گذشته باشد، سطح پایدار پذیرفته می‌شود.
7. فقط در صورت تغییر نسبت به وضعیت پایدار قبلی event ساخته می‌شود.

هر دکمه state و debounce مستقل دارد. handler مشترک فقط pin دارای interrupt را ثبت می‌کند.
این روش non-blocking است؛ ISR کوتاه می‌ماند و main loop با `next_input_event()`
debounce هر دو دکمه را پیش می‌برد.

### صف event

eventهای پذیرفته‌شده داخل یک ring buffer ثابت با ظرفیت ۸ قرار می‌گیرند. این صف heap ندارد و با `critical_section` بین ISR و main محافظت می‌شود.

اگر صف کاملاً پر شود، event جدید فعلاً کنار گذاشته می‌شود. برای دو دکمه و main loop سریع این حالت بعید است، اما در آینده بهتر است overflow counter یا سیاست مشخص‌تری اضافه شود.

## ۹. event loop فعلی

انتهای `main` دیگر یک loop کاملاً خالی نیست:

```rust
loop {
    while let Some(event) = io::next_input_event() {
        match event {
            InputEvent::ButtonPressed(ButtonId::Backlight) => {
                screen.toggle_backlight();
            }
            InputEvent::ButtonPressed(ButtonId::LedMode) => {
                leds.cycle_mode();
            }
            InputEvent::ButtonReleased(button) => log(button),
        }
    }

    leds.update();
    Timer::after(Duration::from_millis(1)).await;
}
```

دو مسیر event فعلی چنین‌اند:

```text
GPIO32 → interrupt → debounce → event queue → LCD backlight
GPIO27 → interrupt → debounce → event queue → LedController
```

وقتی UI ساخته شود، همین match می‌تواند eventها را به menu، file manager یا application فعال تحویل دهد.

## ۱۰. PWM

ماژول `src/pwm` timer و channel مربوط به LEDC را می‌سازد.
ماژول `src/leds.rs` لایهٔ رفتاری بالاتر است و حالت‌های Off، Blink و Fade،
state مربوط به fade و زمان آخرین update را نگه می‌دارد.

`PwmController::set_duty` فقط duty cycle را تنظیم می‌کند:

```rust
pwm.set_duty(50);
```

این تابع عمداً delay ندارد. اگر caller بخواهد duty برای مدتی ثابت بماند، خودش زمان‌بندی را انجام می‌دهد:

```rust
pwm.set_duty(50);
// delay یا scheduler در لایهٔ caller
pwm.off();
```

این جداسازی باعث می‌شود درایور PWM ناخواسته CPU و event loop را متوقف نکند.

## ۱۱. timer

ماژول `src/timer` یک `PeriodicTimer` می‌سازد. handler وقفه را پاک می‌کند و یک counter اتمیک را افزایش می‌دهد.

این timer فعلاً یک abstraction عمومی scheduler یا task runtime نیست؛ فقط یک نمونهٔ ساده برای فهم interrupt دوره‌ای و شمارندهٔ اتمیک است.

## ۱۲. UART و گزارش خطا

UART0 با GPIO1 و GPIO3 ساخته می‌شود. در boot، نتیجهٔ initialization نمایشگر، mount کارت و eventهای دکمه روی UART نوشته می‌شوند.

UART در این پروژه ابزار اصلی مشاهدهٔ رفتار داخلی است. روی نمایشگر فقط پیام‌های مناسب کاربر نمایش داده می‌شود؛ جزئیات فنی خطا معمولاً روی UART قرار می‌گیرند.

## ۱۳. شبکه و تنظیمات محلی

SSID و رمز Wi-Fi در repository نگهداری نمی‌شوند. آن‌ها هنگام build از environment خوانده می‌شوند:

```bash
ESP_START_WIFI_SSID='your-ssid' ESP_START_WIFI_PASSWORD='your-password' cargo build
```

اگر `ESP_START_WIFI_SSID` تنظیم نشده باشد، boot شبکه را رد می‌کند و سیستم offline ادامه می‌دهد. اتصال Wi-Fi، دریافت DHCP و اتصال TCP timeout صریح دارند. HTTP client پس از قطع اتصال با فاصلهٔ زمانی محدود دوباره تلاش می‌کند.

bufferهای RX/TX فعلاً `StaticCell`های singleton هستند؛ بنابراین پیاده‌سازی کنونی عمداً تنها یک TCP socket می‌سازد. قبل از پشتیبانی از چند network application باید pool یا مالکیت جداگانهٔ bufferها طراحی شود. HTTP parser کامل، TLS و مدیریت چند request هنوز پیاده‌سازی نشده‌اند.

## ۱۴. نقشهٔ فایل‌ها

| مسیر | مسئولیت |
|---|---|
| `src/bin/main.rs` | boot، اتصال subsystemها و event loop |
| `src/spi_bus.rs` | ساخت SPI2 مشترک |
| `src/screen/` | درایور، framebuffer، graphics و text |
| `src/sd/` | SD card، FAT volume و time source |
| `src/io/` | pin config، GPIO interrupt، debounce و input events |
| `src/leds.rs` | state machine مربوط به Off، Blink و Fade |
| `src/pwm/` | تنظیم LEDC PWM |
| `src/timer/` | timer دوره‌ای و شمارنده |
| `src/net/` | Wi-Fi network stack، TCP client و polling پاسخ |
| `src/com/uart.rs` | UART0 |
| `src/utils.rs` | delay سادهٔ busy-wait |
| `src/lib.rs` | export ماژول‌های کتابخانه |

## ۱۵. build و اجرا

بررسی قالب‌بندی:

```bash
cargo fmt --check
```

بررسی typeها و compilation بدون لینک نهایی:

```bash
cargo check
```

بررسی warningها:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

ساخت firmware:

```bash
cargo build
```

فلش و monitor:

```bash
espflash flash --monitor target/xtensa-esp32-none-elf/debug/esp-start
```

## ۱۶. اصول ادامهٔ توسعه

هنگام تغییر پروژه این قواعد مفیدند:

1. ابتدا correctness و پایداری سخت‌افزار را حفظ کن.
2. SPI مشترک و CS مستقل دستگاه‌ها را دست‌کم نگیر.
3. drawing را از hardware I/O جدا نگه دار.
4. داخل ISR کار طولانی، allocation، log سنگین یا delay انجام نده.
5. خطای SD را با نبودن کارت یکی نکن.
6. timing را در caller یا scheduler نگه دار، نه داخل setterهای سخت‌افزار.
7. قبل از استفاده از مثال‌های اینترنتی، نسخه‌های واقعی `Cargo.toml` و `Cargo.lock` را بررسی کن.
8. بعد از هر تغییر `fmt`، `check` و `clippy` را اجرا کن.

## ۱۷. محدودیت‌ها و قدم‌های بعدی

مواردی که هنوز عمداً ساده هستند:

- `flush()` کل framebuffer را می‌فرستد.
- event queue فقط ۸ خانه دارد و overflow report ندارد.
- فعلاً دو دکمهٔ اختصاصی تعریف شده‌اند و هنوز navigation کامل نداریم.
- timestamp مربوط به SD ثابت و جعلی است.
- main loop هنوز sleep یا power management ندارد.
- UI manager و routing رویداد میان applicationها ساخته نشده است.
- keyboard نهایی هنوز انتخاب نشده است.
- شبکه فعلاً یک TCP socket با bufferهای singleton دارد و HTTP client هنوز parser کامل یا TLS ندارد.
- `MountedSd` در boot پس از root listing drop می‌شود و file manager باید آن را در state برنامه نگه دارد.

قدم‌های منطقی آینده:

1. نمایش یک menu ساده با دکمه.
2. تعریف actionهایی مثل Up، Down، Select و Back.
3. جداکردن event dispatch از `main` وقتی تعداد screenها بیشتر شد.
4. ساخت file browser روی `MountedSd`.
5. اضافه‌کردن overflow telemetry برای input queue.
6. بررسی dirty-bank flushing فقط زمانی که نیاز عملکردی واقعی دیده شد.
