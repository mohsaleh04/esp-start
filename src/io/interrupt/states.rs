use crate::io::{ButtonId, InputEvent};
use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use critical_section::{CriticalSection, Mutex};
use esp_hal::gpio::{Event, Input};

const EVENT_QUEUE_CAPACITY: usize = 8;

struct EventQueue {
    events: [Option<InputEvent>; EVENT_QUEUE_CAPACITY],
    read: usize,
    write: usize,
    len: usize,
}

impl EventQueue {
    const fn new() -> Self {
        Self {
            events: [None; EVENT_QUEUE_CAPACITY],
            read: 0,
            write: 0,
            len: 0,
        }
    }

    fn push(&mut self, event: InputEvent) {
        if self.len == EVENT_QUEUE_CAPACITY {
            return;
        }

        self.events[self.write] = Some(event);
        self.write = (self.write + 1) % EVENT_QUEUE_CAPACITY;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<InputEvent> {
        if self.len == 0 {
            return None;
        }

        let event = self.events[self.read].take();
        self.read = (self.read + 1) % EVENT_QUEUE_CAPACITY;
        self.len -= 1;
        event
    }
}

pub(super) static PRIMARY_BUTTON: Mutex<RefCell<Option<Input<'static>>>> =
    Mutex::new(RefCell::new(None));
static PRIMARY_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
static EVENT_QUEUE: Mutex<RefCell<EventQueue>> = Mutex::new(RefCell::new(EventQueue::new()));

pub(super) fn primary_button_is_pressed() -> bool {
    PRIMARY_BUTTON_PRESSED.load(Ordering::Relaxed)
}

pub(super) fn record_primary_button_state(cs: CriticalSection<'_>, pressed: bool) {
    let was_pressed = PRIMARY_BUTTON_PRESSED.swap(pressed, Ordering::Relaxed);
    if was_pressed == pressed {
        return;
    }

    let event = if pressed {
        InputEvent::ButtonPressed(ButtonId::Primary)
    } else {
        InputEvent::ButtonReleased(ButtonId::Primary)
    };
    EVENT_QUEUE.borrow_ref_mut(cs).push(event);
}

pub(super) fn init_primary_button(mut button: Input<'static>) {
    critical_section::with(|cs| {
        PRIMARY_BUTTON_PRESSED.store(button.is_low(), Ordering::Relaxed);
        button.listen(Event::AnyEdge);
        PRIMARY_BUTTON.borrow_ref_mut(cs).replace(button);
    });
}

pub(super) fn next_event() -> Option<InputEvent> {
    critical_section::with(|cs| EVENT_QUEUE.borrow_ref_mut(cs).pop())
}
