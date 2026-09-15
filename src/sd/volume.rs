use crate::sd::time::FixedTimeSource;
use core::ops::ControlFlow;
use embedded_hal::{delay::DelayNs, spi::SpiDevice};
use embedded_sdmmc::{
    DirEntry, Error as FilesystemError, SdCard, SdCardError, Volume, VolumeIdx, VolumeManager,
};

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

    pub fn mount(&self) -> Result<MountedSd<'_, SPI, DELAY>, SdStorageError> {
        let size_bytes = self
            .manager
            .device(|card| card.num_bytes())
            .map_err(SdStorageError::from_card)?;
        let volume = self
            .manager
            .open_volume(VolumeIdx(0))
            .map_err(SdStorageError::from_filesystem)?;

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
