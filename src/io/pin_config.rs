use esp_hal::gpio::{InputConfig, Pull};

pub enum PinConfig {
    PullUp,
    PullDown,
    Floating,
}

impl PinConfig {
    pub fn as_input(&self) -> InputConfig {
        match self {
            Self::PullUp => InputConfig::default().with_pull(Pull::Up),
            Self::PullDown => InputConfig::default().with_pull(Pull::Down),
            Self::Floating => InputConfig::default().with_pull(Pull::None),
        }
    }
}
