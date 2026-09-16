use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};

pub struct ScreenPins {
    pub backlight: Output<'static>,
    pub rst: Output<'static>,
    pub dc: Output<'static>,
}

pub struct ScreenOutPins {
    pub screen: ScreenPins,
    pub cs: Output<'static>,
}

impl ScreenOutPins {
    pub fn new(
        backlight: impl OutputPin + 'static,
        rst: impl OutputPin + 'static,
        dc: impl OutputPin + 'static,
        cs: impl OutputPin + 'static,
    ) -> Self {
        Self {
            screen: ScreenPins {
                backlight: Output::new(backlight, Level::Low, OutputConfig::default()),
                rst: Output::new(rst, Level::High, OutputConfig::default()),
                dc: Output::new(dc, Level::Low, OutputConfig::default()),
            },
            cs: Output::new(cs, Level::High, OutputConfig::default()),
        }
    }
}
