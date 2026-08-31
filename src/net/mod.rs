pub mod runner;
pub mod socket;

use core::fmt::Write;
use embassy_net::driver::Driver;
use embassy_net::{Config, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer as NetTimer};
use esp_hal::Blocking;
use esp_hal::uart::Uart;
use static_cell::StaticCell;
use crate::utils::generate_seed;

const SOCKET_COUNT: usize = 3;

static RESOURCES: StaticCell<StackResources<SOCKET_COUNT>> = StaticCell::new();

pub fn setup<D>(driver: D) -> (Stack<'static>, Runner<'static, D>)
    where D: Driver + 'static
{
    let net_config = Config::dhcpv4(Default::default());
    let resources = RESOURCES.init(StackResources::new());
    let seed = generate_seed();

    embassy_net::new(driver, net_config, resources, seed)
}

pub async fn wait_for_config_up(stack: Stack<'static>, uart: &mut Uart<'static, Blocking>) {
    while !stack.is_config_up() {
        uart.write_str("Waiting for DHCP...\r\n").unwrap();
        NetTimer::after(Duration::from_millis(500)).await;
    }
}
