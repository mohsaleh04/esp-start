use embassy_executor::Spawner;
use esp_hal::peripherals::BT;
use crate::bluetooth::scanner::run;

#[embassy_executor::task]
async fn bluetooth_scanner_task(preph: BT<'static>) {
    run(preph).await;
}

pub fn run_bt_scan_task(spawner: &Spawner, preph: BT<'static>) {
    spawner.spawn(bluetooth_scanner_task(preph).expect("Couldn't create BT Scanner Task!\r\n"));
}
