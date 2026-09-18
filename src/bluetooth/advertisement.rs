use core::cell::RefCell;
use bt_hci::param::{BdAddr, LeAdvReportsIter};
use heapless::Deque;
use trouble_host::prelude::EventHandler;
use crate::bluetooth::scanner::ScannerResult;

pub(super) struct AdvertisementHandler {
    seen: RefCell<Deque<BdAddr, 128>>,
}

impl AdvertisementHandler {
    pub(super) fn new() -> Self {
        Self {
            seen: RefCell::new(Deque::new())
        }
    }
}

impl EventHandler for AdvertisementHandler {
    fn on_adv_reports(&self, mut reports: LeAdvReportsIter<'_>) {
        let mut seen = self.seen.borrow_mut();

        while let Some(Ok(report)) = reports.next() {
            if seen
                .iter()
                .all(|address| address.raw() != report.addr.raw())
            {
                crate::bluetooth::scanner::push_scan_result(ScannerResult {
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
