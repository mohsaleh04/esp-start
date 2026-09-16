use crate::io::interrupt::debounce::ButtonDebounce;
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
static PRIMARY_BUTTON_DEBOUNCE: Mutex<RefCell<ButtonDebounce>> =
    Mutex::new(RefCell::new(ButtonDebounce::new()));
static EVENT_QUEUE: Mutex<RefCell<EventQueue>> = Mutex::new(RefCell::new(EventQueue::new()));

pub(super) fn primary_button_is_pressed() -> bool {
    process_debounce();
    PRIMARY_BUTTON_PRESSED.load(Ordering::Relaxed)
}

fn process_debounce() {
    critical_section::with(|cs| {
        let event = PRIMARY_BUTTON_DEBOUNCE.borrow_ref_mut(cs).take_event();
        if let Some(event) = event {
            let pressed = matches!(event, InputEvent::ButtonPressed(ButtonId::Primary));
            PRIMARY_BUTTON_PRESSED.store(pressed, Ordering::Relaxed);
            EVENT_QUEUE.borrow_ref_mut(cs).push(event);
        }
    });
}

pub(super) fn record_primary_button_edge(cs: CriticalSection<'_>, pressed: bool) {
    PRIMARY_BUTTON_DEBOUNCE
        .borrow_ref_mut(cs)
        .record_edge(pressed);
}

pub(super) fn init_primary_button(mut button: Input<'static>) {
    critical_section::with(|cs| {
        let pressed = button.is_low();
        PRIMARY_BUTTON_PRESSED.store(pressed, Ordering::Relaxed);
        PRIMARY_BUTTON_DEBOUNCE
            .borrow_ref_mut(cs)
            .configure(pressed);
        button.listen(Event::AnyEdge);
        PRIMARY_BUTTON.borrow_ref_mut(cs).replace(button);
    });
}

pub(super) fn next_event() -> Option<InputEvent> {
    process_debounce();
    critical_section::with(|cs| EVENT_QUEUE.borrow_ref_mut(cs).pop())
}
