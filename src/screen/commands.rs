#[repr(u8)]
pub(super) enum ScreenCommand {
    FunctionSet = 0x20,
    ExtendedFunctionSet = (ScreenCommand::FunctionSet as u8) | 0x01,

    SetContrast = 0x80,
    SetTempCoeff = 0x04,
    SetBias = 0x10,

    NormalDisplayMode = 0x0C,
    PixelTestMode = 0x09,
}

#[repr(u8)]
pub(super) enum AddressingCommand {
    Horizontal = (ScreenCommand::FunctionSet as u8) | 0x00,
    Vertical = (ScreenCommand::FunctionSet as u8) | 0x02,
}

#[repr(u8)]
pub(super) enum PositioningCommand {
    SetX = 0x80,
    SetBank = 0x40  // Selection data bank in display memory
}

pub(crate) mod config {
    pub const DEFAULT_CONTRAST: u8 = 0x36;
    pub(crate) const DEFAULT_BIAS: u8 = 0x03;
    pub(crate) const MAX_CONTRAST: u8 = 0x7F;
}
