use core::cell::RefCell;
use critical_section::Mutex;
use heapless::Deque;

#[derive(Debug, Clone, Copy)]
pub enum BluetoothEvent {
    Connected,
    Disconnected,
}

const EVENT_QUEUE_SIZE: usize = 8;

static EVENT_QUEUE: Mutex<RefCell<Deque<BluetoothEvent, EVENT_QUEUE_SIZE>>> =
    Mutex::new(RefCell::new(Deque::new()));

pub fn push(event: BluetoothEvent) {
    critical_section::with(|cs| {
        let mut queue = EVENT_QUEUE.borrow_ref_mut(cs);

        if queue.is_full() {
            queue.pop_front();
        }
        queue.push_back(event).unwrap();
    });
}

pub fn next() -> Option<BluetoothEvent> {
    critical_section::with(|cs| EVENT_QUEUE.borrow_ref_mut(cs).pop_front())
}
