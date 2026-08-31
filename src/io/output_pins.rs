use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};

pub struct OutputPins {
    pub test_led: Output<'static>,
    pub blink_led: Output<'static>,
}

impl OutputPins {
    pub fn new(blink_led: impl OutputPin + 'static, test_led: impl OutputPin + 'static) -> Self {
        Self {
            blink_led: Output::new(blink_led, Level::Low, OutputConfig::default()),
            test_led: Output::new(test_led, Level::Low, OutputConfig::default()),
        }
    }
}
