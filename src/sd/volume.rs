use core::fmt::Write;
use crate::sd::time::FixedTimeSource;
use core::ops::ControlFlow;
use embedded_hal::{delay::DelayNs, spi::SpiDevice};
use embedded_sdmmc::{
    DirEntry, Error as FilesystemError, SdCard, SdCardError, Volume, VolumeIdx, VolumeManager,
};
use esp_hal::Blocking;
use esp_hal::uart::Uart;

pub struct SdStorage<SPI, DELAY>
where
    SPI: SpiDevice<u8>,
    DELAY: DelayNs,
{
    manager: VolumeManager<SdCard<SPI, DELAY>, FixedTimeSource>,
}

#[derive(Debug)]
pub enum SdStorageError {
    CardNotFound,
    Card(SdCardError),
    Filesystem(FilesystemError<SdCardError>),
}

impl SdStorageError {
    fn from_card(error: SdCardError) -> Self {
        match error {
            SdCardError::CardNotFound => Self::CardNotFound,
            error => Self::Card(error),
        }
    }

    fn from_filesystem(error: FilesystemError<SdCardError>) -> Self {
        match error {
            FilesystemError::DeviceError(SdCardError::CardNotFound) => Self::CardNotFound,
            error => Self::Filesystem(error),
        }
    }
}

pub struct MountedSd<'a, SPI, DELAY>
where
    SPI: SpiDevice<u8>,
    DELAY: DelayNs,
{
    volume: Volume<'a, SdCard<SPI, DELAY>, FixedTimeSource, 4, 4, 1>,
    size_bytes: u64,
}

impl<SPI, DELAY> SdStorage<SPI, DELAY>
where
    SPI: SpiDevice<u8>,
    DELAY: DelayNs,
{
    pub fn new(spi: SPI, delay: DELAY) -> Self {
        Self {
            manager: VolumeManager::new(SdCard::new(spi, delay), FixedTimeSource),
        }
    }

    pub fn mount(&self, uart: &mut Uart<Blocking>) -> Result<MountedSd<'_, SPI, DELAY>, SdStorageError> {
        uart.write_str("SD: starting card initialization...\r\n").unwrap();
        let size_bytes = match self.manager.device(|card| card.num_bytes()) {
            Ok(size) => {
                writeln!(uart, "SD: card detected, size = {} bytes", size).unwrap();
                size
            }
            Err(error) => {
                writeln!(uart, "SD: card initialization failed: {:?}", error).unwrap();
                return Err(SdStorageError::from_card(error));
            }
        };

        uart.write_str("SD: opening FAT volume...\r\n").unwrap();
        let volume = match self.manager.open_volume(VolumeIdx(0)) {
            Ok(volume) => {
                uart.write_str("SD: FAT volume opened successfully\r\n").unwrap();
                volume
            }
            Err(error) => {
                writeln!(uart, "SD: FAT volume failed: {:?}", error).unwrap();
                return Err(SdStorageError::from_filesystem(error));
            }
        };

        Ok(MountedSd { volume, size_bytes })
    }
}

impl<SPI, DELAY> MountedSd<'_, SPI, DELAY>
where
    SPI: SpiDevice<u8>,
    DELAY: DelayNs,
{
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn for_each_root_entry<F>(&self, visitor: F) -> Result<(), SdStorageError>
    where
        F: FnMut(&DirEntry) -> ControlFlow<()>,
    {
        let root = self
            .volume
            .open_root_dir()
            .map_err(SdStorageError::from_filesystem)?;
        root.iterate_dir(visitor)
            .map_err(SdStorageError::from_filesystem)
    }
}
