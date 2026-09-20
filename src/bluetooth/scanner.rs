use core::{future, cell::RefCell};
use bt_hci::cmd::le::LeSetScanParams;
use bt_hci::controller::ControllerCmdSync;
use critical_section::Mutex;
use embassy_time::Duration;
use heapless::Deque;
use trouble_host::prelude::*;

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

pub(super) async fn run<C>(central: Central<'_, C, DefaultPacketPool>)
where C: Controller + ControllerCmdSync<LeSetScanParams> {
    let mut scanner = Scanner::new(central);
    let mut config = ScanConfig::default();

    config.active = true;
    config.phys = PhySet::M1;
    config.interval = Duration::from_secs(1);
    config.window = Duration::from_secs(1);
    let _session = scanner.scan(&config).await.unwrap();

    future::pending::<()>().await;
}
