use crate::bluetooth::DEFAULT_DISCOVERABLE_NAME;
use core::future;
use trouble_host::Controller;
use trouble_host::peripheral::Peripheral;
use trouble_host::prelude::{
    AdStructure, Advertisement, DefaultPacketPool, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
};

pub(super) async fn run<C>(mut peripheral: Peripheral<'_, C, DefaultPacketPool>)
where C: Controller {
    let mut adv_data = [0u8; 32];
    let ad_struct = [
        AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
        AdStructure::CompleteLocalName(DEFAULT_DISCOVERABLE_NAME)
    ];
    let adv_data_len = AdStructure::encode_slice(&ad_struct, &mut adv_data)
        .expect("Failed to encode BLE advertisement");

    let advertiser = peripheral.advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data[..adv_data_len],
                scan_data: &[],
            },
        ).await.expect("Failed to start BLE advertising");
    let connection = advertiser.accept()
        .await.expect("Failed to accept BLE connection");

    // if connection.is_connected() {
    //     connection.disconnect();
    // }

    future::pending::<()>().await;
}
