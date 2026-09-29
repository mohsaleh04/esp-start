use embassy_time::Timer;
use trouble_host::prelude::PacketPool;
use trouble_host::gatt::GattConnection;
use crate::bluetooth::gatt::EspStartGattServer;

pub(super) async fn notify_task<P: PacketPool>(
    server: &EspStartGattServer<'_>,
    conn: &GattConnection<'_, '_, P>,
) {
    let status = server.esp_start.status;

    let mut counter: u8 = 0;

    loop {
        counter = counter.wrapping_add(1);

        let mut value = [0u8; 32];
        value[0] = counter;

        if status.notify(conn, &value, true).await.is_err() {
            break;
        }

        Timer::after_secs(2).await;
    }
}
