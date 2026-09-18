/*
 * Missing ESP32 Bluetooth ROM symbols required by libbtdm_app.
 *
 * Addresses are taken from Espressif's official ESP32 ROM linker script.
 */

PROVIDE(ld_iscan_evt_start_cbk = 0x4003b58c);
PROVIDE(ld_page_evt_start_cbk  = 0x4003cf40);
PROVIDE(ld_pscan_evt_start_cbk = 0x4003e924);
