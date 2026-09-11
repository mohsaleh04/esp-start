use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};

pub struct SdOutPins {
    pub cs: Output<'static>,
}

impl SdOutPins {
    pub fn new(
        sd_cs: impl OutputPin + 'static,
    ) -> Self {
        Self {
            cs: Output::new(sd_cs, Level::High, OutputConfig::default()),
        }
    }
}
