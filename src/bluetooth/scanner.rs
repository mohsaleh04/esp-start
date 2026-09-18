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

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

#[derive(Clone, Copy, Debug)]
pub struct ScannerResult {
    pub address: BdAddr,
}

static RESULTS: Mutex<RefCell<Deque<ScannerResult, 32>>> = Mutex::new(RefCell::new(Deque::new()));

pub fn next_scan_result() -> Option<ScannerResult> {
    critical_section::with(|cs| RESULTS.borrow_ref_mut(cs).pop_front())
}

fn push_scan_result(result: ScannerResult) {
    critical_section::with(|cs| {
        let mut queue = RESULTS.borrow_ref_mut(cs);
        if queue.is_full() {
            queue.pop_front();
        }

        let _ = queue.push_back(result);
    });
}

pub(super) async fn run(bluetooth: BT<'static>) {
    let connector = BleConnector::new(bluetooth, Default::default()).unwrap();
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
    let handler = AdvertisementHandler {
        seen: RefCell::new(Deque::new()),
    };

    let mut scanner = Scanner::new(central);
    let _ = join(runner.run_with_handler(&handler), async {
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

struct AdvertisementHandler {
    seen: RefCell<Deque<BdAddr, 128>>,
}

impl EventHandler for AdvertisementHandler {
    fn on_adv_reports(&self, mut reports: LeAdvReportsIter<'_>) {
        let mut seen = self.seen.borrow_mut();

        while let Some(Ok(report)) = reports.next() {
            if seen
                .iter()
                .all(|address| address.raw() != report.addr.raw())
            {
                push_scan_result(ScannerResult {
                    address: report.addr,
                });

                if seen.is_full() {
                    seen.pop_front();
                }

                let _ = seen.push_back(report.addr);
            }
        }
    }
}
