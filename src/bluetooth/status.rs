use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    signal::Signal,
};

#[derive(Debug, Clone, Copy)]
pub enum BluetoothStatus {
    Idle,
    Ready,
    Failed,
    Playing,
    Paused,
}

static STATUS_SIGNAL: Signal<CriticalSectionRawMutex, BluetoothStatus> = Signal::new();

pub fn set(status: BluetoothStatus) {
    STATUS_SIGNAL.signal(status);
}

pub(super) async fn wait() -> BluetoothStatus {
    STATUS_SIGNAL.wait().await
}
