use core::cell::RefCell;
use bt_hci::param::{BdAddr, LeAdvReportsIter};
use heapless::Deque;
use trouble_host::prelude::EventHandler;
use crate::bluetooth::scanner;
use crate::bluetooth::scanner::ScannerResult;

pub struct AdvertisementHandler {
    seen: RefCell<Deque<BdAddr, 128>>,
}

impl AdvertisementHandler {
    pub fn new() -> Self {
        Self {
            seen: RefCell::new(Deque::new())
        }
    }
}

impl EventHandler for AdvertisementHandler {
    fn on_adv_reports(&self, mut reports: LeAdvReportsIter) {
        let mut seen = self.seen.borrow_mut();

        while let Some(Ok(report)) = reports.next() {
            let mut data = [0u8; 32];
            for (i, d) in report.data.iter().enumerate() {
                data[i] = *d;
            }
            scanner::push_scan_result(ScannerResult {
                address: report.addr,
                rssi: report.rssi,
                data,
                data_len: report.data.len()
            });

            if seen.is_full() {
                seen.pop_front();
            }

            seen.push_back(report.addr).unwrap();
        }
    }
}
