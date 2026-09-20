#[derive(Debug, Clone)]
pub struct AdvertisementData {
    pub flags: Option<u8>,
    pub local_name: Option<heapless::String<32>>,
    pub tx_power: Option<i8>,
}

impl AdvertisementData {
    pub(super) fn default() -> Self {
        Self {
            flags: None,
            local_name: None,
            tx_power: None
        }
    }

    pub fn merge(&mut self, other: AdvertisementData) -> bool {
        let mut changed = false;
        if let Some(flags) = other.flags {
            self.flags = Some(flags);
            changed = true;
        }

        if let Some(name) = other.local_name {
            self.local_name = Some(name);
            changed = true;
        }

        if let Some(tx_power) = other.tx_power {
            self.tx_power = Some(tx_power);
            changed = true;
        }

        // for service in other.services {
        //     if !self.services.contains(&service) {
        //         let _ = self.services.push(service);
        //     }
        // }
        changed
    }
}

//////////////

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(super) enum AdvertisementType {
    Flags = 0x01,
    ServicesList = 0x03,
    LocalName = 0x09,
    TxPower = 0x0A,
    ServiceData = 0x16,
    ManufacturerData = 0xFF,
}

impl TryFrom<u8> for AdvertisementType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Flags as u8 => Ok(Self::Flags),
            x if x == Self::ServicesList as u8 => Ok(Self::ServicesList),
            x if x == Self::LocalName as u8 => Ok(Self::LocalName),
            x if x == Self::TxPower as u8 => Ok(Self::TxPower),
            x if x == Self::ServiceData as u8 => Ok(Self::ServiceData),
            x if x == Self::ManufacturerData as u8 => Ok(Self::ManufacturerData),
            _ => Err(()),
        }
    }
}
