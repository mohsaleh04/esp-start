pub(super) struct Config;

impl Config {
    pub(super) const DEFAULT_DISCOVERABLE_NAME: &'static [u8] = b"Saleh's esp-start";
    pub(super) const RANDOM_ADDRESS: [u8; 6] = [0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff];

    pub(super) const CONNECTIONS_MAX: usize = 1;
    pub(super) const L2CAP_CHANNELS_MAX: usize = 1;
}
