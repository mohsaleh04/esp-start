use embedded_hal::delay::DelayNs;
use embedded_hal::spi::SpiDevice;
use embedded_sdmmc::{SdCard, VolumeIdx, VolumeManager};
use crate::sd::time::DummyTimeSource;

