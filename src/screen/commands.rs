#[repr(u8)]
pub(super) enum ScreenCommand {
    FunctionSet = 0x20, // PD, V, H => PD = 0 -> active, V  = 0 -> horizontal addressing, H  = 1 -> extended instruction set
    SetContrast = 0x80,
    SetTempCoeff = 0x04,
    SetBias = 0x10,
    NormalDisplayMode = 0x0C,
    _PixelTestMode = 0x09,
}

#[repr(u8)]
pub(super) enum PositionCommand {
    SetX = 0x80,
    SetBankY = 0x40
}
