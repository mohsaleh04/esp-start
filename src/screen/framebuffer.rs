use crate::screen::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub(super) const SCREEN_BANKS: usize = SCREEN_HEIGHT / 8;
pub(super) const FRAMEBUFFER_LEN: usize = SCREEN_WIDTH * SCREEN_BANKS;

pub(super) struct Framebuffer {
    bytes: [u8; FRAMEBUFFER_LEN],
}

impl Framebuffer {
    pub(super) const fn new() -> Self {
        Self {
            bytes: [0; FRAMEBUFFER_LEN],
        }
    }

    pub(super) fn clear(&mut self) {
        self.bytes.fill(0);
    }

    pub(super) fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        let index = x + (y / 8) * SCREEN_WIDTH;
        let mask = 1 << (y % 8);

        if on {
            self.bytes[index] |= mask;
        } else {
            self.bytes[index] &= !mask;
        }
    }

    pub(super) fn as_bytes(&self) -> &[u8; FRAMEBUFFER_LEN] {
        &self.bytes
    }
}
