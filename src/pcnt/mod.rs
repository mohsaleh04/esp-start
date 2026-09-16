use esp_hal::gpio::interconnect::InputSignal;
use esp_hal::gpio::{Input, InputConfig, InputPin};
use esp_hal::pcnt::channel::{Channel, EdgeMode};

pub fn setup<const UNIT_NUM: usize, const CH_NUM: usize>(
    channel: &Channel<'static, UNIT_NUM, CH_NUM>,
    trigger_input: impl InputPin + 'static,
    input_config: InputConfig,
    raising_input_mod: EdgeMode,
    falling_input_mod: EdgeMode,
) {
    let input = Input::new(trigger_input, input_config);
    let signal = input.peripheral_input();

    setup_channel(channel, signal, raising_input_mod, falling_input_mod);
}

fn setup_channel<const UNIT_NUM: usize, const CH_NUM: usize>(
    channel: &Channel<'static, UNIT_NUM, CH_NUM>,
    input_signal: InputSignal,
    raising_input_mod: EdgeMode,
    falling_input_mod: EdgeMode,
) {
    channel.set_edge_signal(input_signal);
    channel.set_input_mode(raising_input_mod, falling_input_mod);
}
