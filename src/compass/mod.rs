use embedded_hal::i2c::I2c;

const HMC_ADDR: u8 = 0x1E;
const QMC_ADDR: u8 = 0x0D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompassKind {
    Hmc5883l,
    Qmc5883l,
}

#[derive(Debug, Clone, Copy)]
pub struct MagneticField {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl CompassKind {
    pub fn detect<I: I2c>(i2c: &mut I) -> Option<Self> {
        let mut id = [0u8; 3];

        if i2c.write_read(HMC_ADDR, &[0x0A], &mut id).is_ok()
            && id == *b"H43"
        {
            return Some(Self::Hmc5883l);
        }

        let mut id = [0u8; 1];

        if i2c.write_read(QMC_ADDR, &[0x0D], &mut id).is_ok()
            && id[0] == 0xFF
        {
            return Some(Self::Qmc5883l);
        }

        None
    }

    pub fn init<I: I2c>(&self, i2c: &mut I) -> Result<(), I::Error> {
        match self {
            Self::Hmc5883l => {
                i2c.write(HMC_ADDR, &[0x00, 0x70])?;
                i2c.write(HMC_ADDR, &[0x01, 0x20])?;
                i2c.write(HMC_ADDR, &[0x02, 0x00])?;
            }
            Self::Qmc5883l => {
                i2c.write(QMC_ADDR, &[0x0B, 0x01])?;
                i2c.write(QMC_ADDR, &[0x09, 0x0D])?;
            }
        }

        Ok(())
    }

    pub fn read<I: I2c>(
        &self,
        i2c: &mut I,
    ) -> Result<MagneticField, I::Error> {
        let mut data = [0u8; 6];

        match self {
            Self::Hmc5883l => {
                i2c.write_read(HMC_ADDR, &[0x03], &mut data)?;

                Ok(MagneticField {
                    x: i16::from_be_bytes([data[0], data[1]]),
                    z: i16::from_be_bytes([data[2], data[3]]),
                    y: i16::from_be_bytes([data[4], data[5]]),
                })
            }
            Self::Qmc5883l => {
                i2c.write_read(QMC_ADDR, &[0x00], &mut data)?;

                Ok(MagneticField {
                    x: i16::from_le_bytes([data[0], data[1]]),
                    y: i16::from_le_bytes([data[2], data[3]]),
                    z: i16::from_le_bytes([data[4], data[5]]),
                })
            }
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct CompassCalibration {
    pub min: MagneticField,
    pub max: MagneticField,
}

impl CompassCalibration {
    pub fn new(initial: MagneticField) -> Self {
        Self {
            min: initial,
            max: initial,
        }
    }

    pub fn update(&mut self, value: MagneticField) {
        self.min.x = self.min.x.min(value.x);
        self.min.y = self.min.y.min(value.y);
        self.min.z = self.min.z.min(value.z);

        self.max.x = self.max.x.max(value.x);
        self.max.y = self.max.y.max(value.y);
        self.max.z = self.max.z.max(value.z);
    }

    pub fn offset(&self) -> (f32, f32, f32) {
        (
            (self.min.x as f32 + self.max.x as f32) / 2.0,
            (self.min.y as f32 + self.max.y as f32) / 2.0,
            (self.min.z as f32 + self.max.z as f32) / 2.0,
        )
    }
}
