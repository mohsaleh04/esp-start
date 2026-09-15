use embedded_hal::spi::SpiDevice;

pub trait ScreenSpi: SpiDevice<u8> {}

impl<T> ScreenSpi for T where T: SpiDevice<u8> {}
