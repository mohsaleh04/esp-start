use esp_hal::ledc::{
    LowSpeed,
    channel::{Channel, ChannelIFace},
};

pub struct PwmController<'d> {
    channel: Channel<'d, LowSpeed>,
}

impl<'d> PwmController<'d> {
    pub fn new(channel: Channel<'d, LowSpeed>) -> Self {
        Self { channel }
    }

    pub fn set_duty(&mut self, duty_prcnt: u8) {
        self.channel.set_duty(duty_prcnt).unwrap();
    }

    pub fn off(&mut self) {
        self.channel.set_duty(0).unwrap();
    }
}
