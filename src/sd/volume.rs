use crate::sd::time::DummyTimeSource;
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
    manager: VolumeManager<SdCard<SPI, DELAY>, DummyTimeSource>,
}

#[derive(Debug)]
pub enum SdStorageError {
    Card(SdCardError),
    Filesystem(FilesystemError<SdCardError>),
}

impl SdStorageError {
    pub fn is_card_not_found(&self) -> bool {
        matches!(
            self,
            Self::Card(SdCardError::CardNotFound)
                | Self::Filesystem(FilesystemError::DeviceError(SdCardError::CardNotFound))
        )
    }
}

pub struct MountedSd<'a, SPI, DELAY>
where
    SPI: SpiDevice<u8>,
    DELAY: DelayNs,
{
    volume: Volume<'a, SdCard<SPI, DELAY>, DummyTimeSource, 4, 4, 1>,
    size_bytes: u64,
}

impl<SPI, DELAY> SdStorage<SPI, DELAY>
where
    SPI: SpiDevice<u8>,
    DELAY: DelayNs,
{
    pub fn new(spi: SPI, delay: DELAY) -> Self {
        Self {
            manager: VolumeManager::new(SdCard::new(spi, delay), DummyTimeSource),
        }
    }

    pub fn mount(&self) -> Result<MountedSd<'_, SPI, DELAY>, SdStorageError> {
        let size_bytes = self
            .manager
            .device(|card| card.num_bytes())
            .map_err(SdStorageError::Card)?;
        let volume = self
            .manager
            .open_volume(VolumeIdx(0))
            .map_err(SdStorageError::Filesystem)?;

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

    pub fn for_each_root_entry<F>(&self, visitor: F) -> Result<(), FilesystemError<SdCardError>>
    where
        F: FnMut(&DirEntry) -> ControlFlow<()>,
    {
        let root = self.volume.open_root_dir()?;
        root.iterate_dir(visitor)
    }
}
