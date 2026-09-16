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

struct ButtonState {
    button_id: ButtonId,
    input: Mutex<RefCell<Option<Input<'static>>>>,
    pressed: AtomicBool,
    debounce: Mutex<RefCell<ButtonDebounce>>,
}

impl ButtonState {
    const fn new(button_id: ButtonId) -> Self {
        Self {
            button_id,
            input: Mutex::new(RefCell::new(None)),
            pressed: AtomicBool::new(false),
            debounce: Mutex::new(RefCell::new(ButtonDebounce::new())),
        }
    }

    fn init(&self, cs: CriticalSection<'_>, mut input: Input<'static>) {
        let pressed = input.is_low();
        self.pressed.store(pressed, Ordering::Relaxed);
        self.debounce.borrow_ref_mut(cs).configure(pressed);
        input.listen(Event::AnyEdge);
        self.input.borrow_ref_mut(cs).replace(input);
    }

    fn handle_interrupt(&self, cs: CriticalSection<'_>) {
        let mut input = self.input.borrow_ref_mut(cs);
        let Some(input) = input.as_mut() else {
            return;
        };
        if !input.is_interrupt_set() {
            return;
        }

        let pressed = input.is_low();
        input.clear_interrupt();
        self.debounce.borrow_ref_mut(cs).record_edge(pressed);
    }

    fn take_event(&self, cs: CriticalSection<'_>) -> Option<InputEvent> {
        let event = self.debounce.borrow_ref_mut(cs).take_event(self.button_id);
        if let Some(event) = event {
            self.pressed.store(
                matches!(event, InputEvent::ButtonPressed(_)),
                Ordering::Relaxed,
            );
        }
        event
    }

    fn is_pressed(&self) -> bool {
        self.pressed.load(Ordering::Relaxed)
    }
}

static BACKLIGHT_BUTTON: ButtonState = ButtonState::new(ButtonId::Backlight);
static LED_MODE_BUTTON: ButtonState = ButtonState::new(ButtonId::LedMode);
static EVENT_QUEUE: Mutex<RefCell<EventQueue>> = Mutex::new(RefCell::new(EventQueue::new()));

fn button(button_id: ButtonId) -> &'static ButtonState {
    match button_id {
        ButtonId::Backlight => &BACKLIGHT_BUTTON,
        ButtonId::LedMode => &LED_MODE_BUTTON,
    }
}

fn process_debounce() {
    critical_section::with(|cs| {
        let mut queue = EVENT_QUEUE.borrow_ref_mut(cs);
        for button in [&BACKLIGHT_BUTTON, &LED_MODE_BUTTON] {
            if let Some(event) = button.take_event(cs) {
                queue.push(event);
            }
        }
    });
}

pub(super) fn init_button(button_id: ButtonId, input: Input<'static>) {
    critical_section::with(|cs| button(button_id).init(cs, input));
}

pub(super) fn handle_gpio_interrupts() {
    critical_section::with(|cs| {
        BACKLIGHT_BUTTON.handle_interrupt(cs);
        LED_MODE_BUTTON.handle_interrupt(cs);
    });
}

pub(super) fn button_is_pressed(button_id: ButtonId) -> bool {
    process_debounce();
    button(button_id).is_pressed()
}

pub(super) fn next_event() -> Option<InputEvent> {
    process_debounce();
    critical_section::with(|cs| EVENT_QUEUE.borrow_ref_mut(cs).pop())
}
