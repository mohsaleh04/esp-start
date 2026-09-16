use embassy_executor::Spawner;
use embassy_net::Runner;

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, esp_radio::wifi::Interface<'static>>) {
    runner.run().await;
}

pub fn run_wifi_net_task(
    spawner: Spawner,
    runner: Runner<'static, esp_radio::wifi::Interface<'static>>,
) {
    spawner.spawn(net_task(runner).expect("Couldn't create Net Task!\r\n"));
}
