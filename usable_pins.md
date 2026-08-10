## GPIOهایی که روی برد تو باید بشناسی

برای کارهای عمومی، من فعلاً این گروه‌بندی رو تو ذهنت نگه می‌داشتم:

| GPIO                   | وضعیت پیشنهادی                                 |
| ---------------------- | ---------------------------------------------- |
| **18, 19, 21, 22, 23** | 🟢 خیلی مناسب                                  |
| **25, 26, 27, 32, 33** | 🟢 خیلی مناسب                                  |
| **16, 17**             | 🟢 روی WROOM-32D تو قابل استفاده               |
| 4, 13, 14              | 🟡 قابل استفاده، ولی عملکردهای جانبی هم دارند  |
| **0, 2, 5, 12, 15**    | 🟡 Strapping؛ موقع boot مراقب باش              |
| **1, 3**               | 🟡 UART0؛ فعلاً برای serial/flashing آزاد بذار |
| **34, 35, 36, 39**     | 🔵 فقط **Input**                               |
| **6–11**               | 🔴 استفاده نکن؛ متصل به SPI Flash              |

Espressif رسماً GPIOهای `0,2,5,12,15` رو strapping معرفی می‌کنه؛ `34–39` فقط ورودی‌اند و `6–11` برای Flash استفاده می‌شن. ([Espressif Systems][3])

پس برای آزمایش‌های اولیه یه قانون خیلی راحت:

```text
18 19 21 22 23
25 26 27
32 33
```

**این ده‌تا رو منطقه‌ی امن خودت در نظر بگیر.**

و چون الان داریم HAL استفاده می‌کنیم، بعد از اینکه LED روشن شد یه آزمایش خیلی جذاب‌تر هم داریم: یک بار `Output::new()` و `set_high()` رو کنار بذاریم و **مستقیماً رجیستر MMIO مربوط به GPIO23 رو با Rust بنویسیم**. اونجا دقیقاً می‌بینی HAL پشت پرده چه کاری با سخت‌افزار انجام می‌ده.

[1]: https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32/esp32-devkitc/user_guide.html?utm_source=chatgpt.com "ESP32-DevKitC V4 - ESP32 - — esp-dev-kits latest documentation"
[2]: https://docs.espressif.com/projects/rust/esp-hal/1.1.0/esp32c6/esp_hal/gpio/dedicated/fn.write_ll.html?utm_source=chatgpt.com "write_ll in esp_hal::gpio::dedicated - Rust"
[3]: https://docs.espressif.com/projects/arduino-esp32/en/latest/boards/ESP32-DevKitC-1.html?utm_source=chatgpt.com "ESP32-DevKitC-1 - - — Arduino ESP32 latest documentation"
