use crate::bluetooth::gatt::EspStartGattServer;
use crate::bluetooth::status::{BluetoothStatus, wait as status_wait};
use trouble_host::gatt::GattConnection;
use trouble_host::prelude::PacketPool;

pub(super) async fn notify_task<P: PacketPool>(
    server: &EspStartGattServer<'_>,
    conn: &GattConnection<'_, '_, P>,
) {
    let status_char = server.esp_start.status;

    loop {
        let status = status_wait().await;
        let bytes = match status {
            BluetoothStatus::Idle => b"INT-STS-IDLE".as_slice(),
            BluetoothStatus::Ready => b"INT-STS-READY".as_slice(),
            BluetoothStatus::Failed => b"INT-STS-FAILED".as_slice(),
            BluetoothStatus::Playing => b"INT-STS-PLAYING".as_slice(),
            BluetoothStatus::Paused => b"INT-STS-PAUSED".as_slice(),
        };

        let mut value = [0u8; 32];
        value[..bytes.len()].copy_from_slice(bytes);

        if status_char.notify(conn, &value, true).await.is_err() {
            break;
        }
    }
}
