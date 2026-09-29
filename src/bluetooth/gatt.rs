use trouble_host::prelude::{
    FromGatt, GapConfig, PeripheralConfig, appearance, gatt_server, gatt_service,
};

#[gatt_service(uuid = "12345678-1234-1234-1234-123456789001")]
pub struct EspStartService {
    #[characteristic(uuid = "12345678-1234-1234-1234-123456789002", write)]
    pub command: [u8; 32],
    #[characteristic(uuid = "12345678-1234-1234-1234-123456789003", read, notify)]
    pub status: [u8; 32],
}

#[gatt_server]
pub struct EspStartGattServer {
    pub esp_start: EspStartService,
}

pub(super) fn setup_gatt_server(bt_name: &str) -> Result<EspStartGattServer<'_>, &'static str> {
    EspStartGattServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: bt_name,
        appearance: &appearance::MEDIA_PLAYER,
    }))
}
