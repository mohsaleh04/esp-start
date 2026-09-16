use crate::pwm::{self, PwmChannelConfig, PwmController, PwmTimerConfig};
use crate::timer;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};
use esp_hal::ledc::timer::Timer;
use esp_hal::ledc::{
    Ledc, LowSpeed, channel::Number as ChannelNumber, timer::Number as TimerNumber,
};
use esp_hal::peripherals::LEDC;
use static_cell::StaticCell;

pub const UPDATE_INTERVAL_MS: u64 = 300;

static PWM_TIMER: StaticCell<Timer<'static, LowSpeed>> = StaticCell::new();

#[derive(Clone, Copy, Debug)]
pub enum LedMode {
    Off,
    Blink,
    Fade,
}

impl LedMode {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Blink,
            Self::Blink => Self::Fade,
            Self::Fade => Self::Off,
        }
    }
}

struct FadeState {
    duty: u8,
    descending: bool,
    last_timer_count: u32,
}

impl FadeState {
    fn new(timer_count: u32) -> Self {
        Self {
            duty: 0,
            descending: false,
            last_timer_count: timer_count,
        }
    }

    fn reset(&mut self, timer_count: u32) {
        self.duty = 0;
        self.descending = false;
        self.last_timer_count = timer_count;
    }

    fn advance(&mut self) {
        if self.descending {
            if self.duty == 0 {
                self.descending = false;
                self.duty = 10;
            } else {
                self.duty = self.duty.saturating_sub(10);
            }
        } else if self.duty >= 100 {
            self.descending = true;
            self.duty = 90;
        } else {
            self.duty += 10;
        }
    }
}

pub struct LedController {
    mode: LedMode,
    blink: Output<'static>,
    fade_a: PwmController<'static>,
    fade_b: PwmController<'static>,
    last_blink_timer_count: u32,
    fade: FadeState,
}

impl LedController {
    pub fn new(
        ledc_peripheral: LEDC<'static>,
        blink_pin: impl OutputPin + 'static,
        fade_a_pin: impl PeripheralOutput<'static>,
        fade_b_pin: impl PeripheralOutput<'static>,
    ) -> Self {
        let blink = Output::new(blink_pin, Level::Low, OutputConfig::default());

        let mut ledc = Ledc::new(ledc_peripheral);
        let pwm_timer = PWM_TIMER.init(ledc.timer::<LowSpeed>(TimerNumber::Timer0));
        pwm::setup_timer(pwm_timer, PwmTimerConfig::default(500));
        let fade_a = pwm::setup_channel(
            &mut ledc,
            fade_a_pin,
            pwm_timer,
            ChannelNumber::Channel1,
            PwmChannelConfig::default(),
        );
        let fade_b = pwm::setup_channel(
            &mut ledc,
            fade_b_pin,
            pwm_timer,
            ChannelNumber::Channel2,
            PwmChannelConfig::default(),
        );

        let timer_count = timer::event_counter();
        Self {
            mode: LedMode::Off,
            blink,
            fade_a,
            fade_b,
            last_blink_timer_count: timer_count,
            fade: FadeState::new(timer_count),
        }
    }

    pub fn cycle_mode(&mut self) -> LedMode {
        self.mode = self.mode.next();

        self.blink.set_low();
        self.fade_a.off();
        self.fade_b.off();

        let timer_count = timer::event_counter();
        self.last_blink_timer_count = timer_count;
        self.fade.reset(timer_count);
        self.mode
    }

    pub fn update(&mut self) {
        let timer_count = timer::event_counter();
        match self.mode {
            LedMode::Off => {}
            LedMode::Blink => self.update_blink(timer_count),
            LedMode::Fade => self.update_fade(timer_count),
        }
    }

    fn update_blink(&mut self, timer_count: u32) {
        let elapsed_ticks = timer_count.wrapping_sub(self.last_blink_timer_count);
        if elapsed_ticks == 0 {
            return;
        }

        if !elapsed_ticks.is_multiple_of(2) {
            self.blink.toggle();
        }
        self.last_blink_timer_count = timer_count;
    }

    fn update_fade(&mut self, timer_count: u32) {
        let elapsed_ticks = timer_count.wrapping_sub(self.fade.last_timer_count);
        if elapsed_ticks == 0 {
            return;
        }

        for _ in 0..elapsed_ticks {
            self.fade.advance();
        }
        self.fade_a.set_duty(self.fade.duty);
        self.fade_b.set_duty(100 - self.fade.duty);
        self.fade.last_timer_count = timer_count;
    }
}
