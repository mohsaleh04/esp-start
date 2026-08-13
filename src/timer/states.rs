use core::sync::atomic::AtomicU32;

pub(super) static TIMER_COUNTER: AtomicU32 = AtomicU32::new(0);
