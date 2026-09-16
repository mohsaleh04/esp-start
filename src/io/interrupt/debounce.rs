use crate::io::{ButtonId, InputEvent, PRIMARY_BUTTON_DEBOUNCE_MS};
use esp_hal::time::{Duration, Instant};

pub(super) struct ButtonDebounce {
    stable_pressed: bool,
    pending_pressed: bool,
    pending: bool,
    last_edge: Instant,
}

impl ButtonDebounce {
    pub(super) const fn new() -> Self {
        Self {
            stable_pressed: false,
            pending_pressed: false,
            pending: false,
            last_edge: Instant::EPOCH,
        }
    }

    pub(super) fn configure(&mut self, pressed: bool) {
        self.stable_pressed = pressed;
        self.pending_pressed = pressed;
        self.pending = false;
        self.last_edge = Instant::now();
    }

    pub(super) fn record_edge(&mut self, pressed: bool) {
        self.pending_pressed = pressed;
        self.pending = true;
        self.last_edge = Instant::now();
    }

    pub(super) fn take_event(&mut self) -> Option<InputEvent> {
        let debounce_time = Duration::from_millis(PRIMARY_BUTTON_DEBOUNCE_MS);
        if !self.pending || self.last_edge.elapsed() < debounce_time {
            return None;
        }

        self.pending = false;
        if self.stable_pressed == self.pending_pressed {
            return None;
        }

        self.stable_pressed = self.pending_pressed;
        Some(if self.stable_pressed {
            InputEvent::ButtonPressed(ButtonId::Primary)
        } else {
            InputEvent::ButtonReleased(ButtonId::Primary)
        })
    }
}
