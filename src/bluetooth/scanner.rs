use core::cell::RefCell;

use bt_hci::cmd::le::LeCreateConn;
use bt_hci::controller::ControllerCmdAsync;
use bt_hci::{
    cmd::le::LeSetScanParams,
    controller::{ControllerCmdSync, ExternalController},
};
use critical_section::Mutex;
use embassy_futures::join::join;
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::BT;
use esp_radio::ble::controller::BleConnector;
use heapless::Deque;
use trouble_host::prelude::*;
use trouble_host::scan::Scanner;
use crate::bluetooth::adv_handler::AdvertisementHandler;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

static RESULTS: Mutex<RefCell<Deque<ScannerResult, 32>>> = Mutex::new(RefCell::new(Deque::new()));

#[derive(Clone, Copy, Debug)]
pub struct ScannerResult {
    pub address: BdAddr,
    pub rssi: i8,
    pub data_len: usize,
    pub data: [u8; 32],
}

pub fn next_scan_result() -> Option<ScannerResult> {
    critical_section::with(|cs| RESULTS.borrow_ref_mut(cs).pop_front())
}

pub(super) fn push_scan_result(result: ScannerResult) {
    critical_section::with(|cs| {
        let mut queue = RESULTS.borrow_ref_mut(cs);
        if queue.is_full() {
            queue.pop_front();
        }

        queue.push_back(result).expect("Failed to push BT scan result!");
    });
}

pub(super) async fn run(bt: BT<'static>) {
    let connector = BleConnector::new(bt, Default::default()).unwrap();
    let controller: ExternalController<_, 1> = ExternalController::new(connector);

    run_scanner(controller).await;
}

async fn run_scanner<C>(controller: C)
where
    C: Controller + ControllerCmdSync<LeSetScanParams> + ControllerCmdAsync<LeCreateConn>,
{
    let address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);

    let mut resources: HostResources<_, DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX>
        = HostResources::new();

    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(address)
        .build();

    let mut runner = stack.runner();
    let central = stack.central();
    let handler = AdvertisementHandler::new();
    let mut scanner = Scanner::new(central);

    join(runner.run_with_handler(&handler), async {
        let mut config = ScanConfig::default();

        config.active = true;
        config.phys = PhySet::M1;
        config.interval = Duration::from_secs(1);
        config.window = Duration::from_secs(1);
        let _session = scanner.scan(&config).await.unwrap();

        loop {
            Timer::after(Duration::from_secs(1)).await;
        }
    }).await;
}
