use crate::bluetooth::advertisement::AdvertisementData;
use crate::bluetooth::advertisement::models::AdvertisementType;
use crate::bluetooth::scanner::ScannerResult;
pub fn parse_advertisement_data(result: &ScannerResult) -> AdvertisementData {
    let data = &result.data[..result.data_len];
    let mut index = 0;
    let mut adv_data = AdvertisementData::default();

    while index < data.len() {
        let len = data[index] as usize;

        if len == 0 {
            break;
        }

        let end = index + 1 + len;
        if end > data.len() {
            break;
        }

        let ad_type = data[index + 1];
        let value = &data[index + 2..end];

        match AdvertisementType::try_from(ad_type) {
            Ok(AdvertisementType::Flags) => {
                if let Some(&flags) = value.first() {
                    adv_data.flags = Some(flags);
                }
            }
            Ok(AdvertisementType::ServicesList) => {
                // for chunk in value.chunks_exact(2) {
                //     let uuid = u16::from_le_bytes([chunk[0], chunk[1]]);
                //
                //     if !adv_data.services.contains(&uuid) {
                //         let _ = adv_data.services.push(uuid);
                //     }
                // }
            }
            Ok(AdvertisementType::LocalName) => {
                let value = value.strip_suffix(&[0]).unwrap_or(value);

                if let Ok(name) = core::str::from_utf8(value) {
                    let mut local_name = heapless::String::<32>::new();

                    if local_name.push_str(name).is_ok() {
                        adv_data.local_name = Some(local_name);
                    }
                }
            }
            Ok(AdvertisementType::TxPower) => {
                if let Some(&power) = value.first() {
                    adv_data.tx_power = Some(power as i8);
                }
            }
            Ok(AdvertisementType::ServiceData) => {
                // later
            }
            Ok(AdvertisementType::ManufacturerData) => {
                // later
            }
            Err(_) => {}
        }

        index = end;
    }

    adv_data
}
