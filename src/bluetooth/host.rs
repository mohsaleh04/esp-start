use crate::bluetooth::advertisement::handler::AdvertisementHandler;
use crate::bluetooth::{advertiser, scanner};
use bt_hci::cmd::le::LeSetScanParams;
use bt_hci::controller::{ControllerCmdSync, ExternalController};
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either3};
use esp_hal::peripherals::BT;
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::DefaultPacketPool;
use trouble_host::{Address, Controller, HostResources};

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

#[embassy_executor::task]
async fn bluetooth_host_task(bt: BT<'static>) {
    run(bt).await;
}

pub fn run_bt_host(spawner: &Spawner, bt: BT<'static>) {
    spawner.spawn(bluetooth_host_task(bt).expect("Couldn't create Bluetooth Host Task!\r\n"));
}

///////////

pub(super) async fn run(bt: BT<'static>) {
    let connector = BleConnector::new(bt, Default::default()).unwrap();
    let controller: ExternalController<_, 1> = ExternalController::new(connector);

    runner(controller).await;
}

async fn runner<C>(controller: C)
where C: Controller + ControllerCmdSync<LeSetScanParams> {
    let mut resources: HostResources<_, DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();

    let address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(address)
        .build();

    let mut stack_runner = stack.runner();
    let stack_central = stack.central();
    let stack_peripheral = stack.peripheral();
    let handler = AdvertisementHandler::new();

    match select3(
        stack_runner.run_with_handler(&handler),
        scanner::run(stack_central),
        advertiser::run(stack_peripheral)
    ).await {
        Either3::First(result) => {
            result.expect("Bluetooth stack runner failed");
        }
        Either3::Second(_) => {
            panic!("BLE scanner stopped unexpectedly");
        }
        Either3::Third(_) => {
            panic!("BLE advertiser stopped unexpectedly");
        }
    }
}
