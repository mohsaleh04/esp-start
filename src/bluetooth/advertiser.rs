use embassy_futures::select::{select, Either};
use crate::bluetooth::{BluetoothEvent, event, gatt::EspStartGattServer};
use trouble_host::Controller;
use trouble_host::gatt::{GattConnectionEvent, GattEvent};
use trouble_host::peripheral::Peripheral;
use trouble_host::prelude::{
    AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, DefaultPacketPool, LE_GENERAL_DISCOVERABLE,
};
use crate::bluetooth::notifier::notify_task;

pub(super) async fn run<C>(
    mut peripheral: Peripheral<'_, C, DefaultPacketPool>,
    bt_name: &[u8],
    server: &EspStartGattServer<'_>,
) where
    C: Controller,
{
    let mut adv_data = [0u8; 32];
    let ad_struct = [
        AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
        AdStructure::CompleteLocalName(bt_name),
    ];
    let adv_data_len = AdStructure::encode_slice(&ad_struct, &mut adv_data)
        .expect("Failed to encode BLE advertisement");

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data[..adv_data_len],
                scan_data: &[],
            },
        )
        .await
        .expect("Failed to start BLE advertising");

    let connection = advertiser
        .accept()
        .await
        .expect("Failed to accept BLE connection");

    if connection.is_connected() {
        event::push(BluetoothEvent::Connected);
        let gatt_connection = connection.with_attribute_server(server).unwrap();

        let gatt_loop = async {
            loop {
                match gatt_connection.next().await {
                    GattConnectionEvent::Disconnected { .. } => {
                        event::push(BluetoothEvent::Disconnected);
                        break;
                    }
                    GattConnectionEvent::PhyUpdated { .. } => {}
                    GattConnectionEvent::ConnectionParamsUpdated { .. } => {}
                    GattConnectionEvent::RequestConnectionParams(_) => {}
                    GattConnectionEvent::DataLengthUpdated { .. } => {}
                    GattConnectionEvent::FrameSpaceUpdated { .. } => {}
                    GattConnectionEvent::ConnectionRateChanged { .. } => {}
                    GattConnectionEvent::Gatt { event: gatt_event } => {
                        let reply = match gatt_event {
                            GattEvent::Read(read_event) => {
                                if read_event.handle() == server.esp_start.status.handle {
                                    let mut status = [0u8; 32];
                                    status[..7].copy_from_slice(b"ImReady");

                                    if gatt_connection.set(&server.esp_start.status, &status).is_err() {
                                        // error for read generate
                                    }
                                    read_event.accept()
                                } else {
                                    read_event.accept()
                                }
                            }
                            GattEvent::Write(write_event) => {
                                if write_event.handle() == server.esp_start.command.handle {
                                    let mut buffer = [0u8; 512];
                                    write_event.with_data(|offset, data| {
                                        if offset == 0 {
                                            let len = data.len().min(buffer.len());
                                            buffer[..len].copy_from_slice(&data[..len]);
                                            event::push(BluetoothEvent::DataReceived {
                                                len,
                                                data: buffer,
                                            });
                                        }
                                    });
                                }

                                write_event.accept()
                            }
                            other => other.accept(),
                        };

                        match reply {
                            Ok(reply) => {
                                reply.send().await;
                            }
                            Err(_) => {
                                // error for response
                            }
                        }
                    }
                }
            }
        };

        match select(
            gatt_loop,
            notify_task(server, &gatt_connection),
        ).await {
            Either::First(_) => {}
            Either::Second(_) => {}
        }
    }
}
