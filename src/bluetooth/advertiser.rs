use crate::bluetooth::{
    BluetoothEvent, event, gatt::EspStartGattServer, notifier::notify_task, status,
    status::BluetoothStatus,
};
use embassy_futures::select::{Either, select};
use trouble_host::Controller;
use trouble_host::gatt::{GattConnectionEvent, GattEvent};
use trouble_host::peripheral::Peripheral;
use trouble_host::prelude::{
    AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, DefaultPacketPool, LE_GENERAL_DISCOVERABLE,
};

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
                                let text = b"ImReady Hooray";

                                if read_event.handle() == server.esp_start.status.handle
                                    && text.len() <= 32
                                {
                                    let mut status = [0u8; 32];
                                    status[..text.len()].copy_from_slice(text);

                                    if gatt_connection
                                        .set(&server.esp_start.status, &status)
                                        .is_err()
                                    {
                                        status::set(BluetoothStatus::Failed);
                                    } else {
                                        status::set(BluetoothStatus::Ready);
                                    }
                                }
                                read_event.accept()
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

        match select(gatt_loop, notify_task(server, &gatt_connection)).await {
            Either::First(_) => {}
            Either::Second(_) => {}
        }
    }
}
