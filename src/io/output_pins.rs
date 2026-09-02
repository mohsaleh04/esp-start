use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};

pub struct ScreenOutPins {
    pub backlight: Output<'static>,
    pub rst: Output<'static>,
    pub dc: Output<'static>,
    pub cs: Output<'static>,
}

impl ScreenOutPins {
    pub fn new(
        screen_led: impl OutputPin + 'static,
        screen_rst: impl OutputPin + 'static,
        screen_dc: impl OutputPin + 'static,
        screen_cs: impl OutputPin + 'static,
    ) -> Self
    {
        Self {
            backlight: Output::new(screen_led, Level::Low, OutputConfig::default()),
            rst: Output::new(screen_rst, Level::High, OutputConfig::default()),
            dc: Output::new(screen_dc, Level::Low, OutputConfig::default()),
            cs: Output::new(screen_cs, Level::High, OutputConfig::default()),
        }
    }
}
