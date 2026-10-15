use crate::utils::generate_seed;
use embassy_net::driver::Driver;
use embassy_net::{Config, Runner, Stack, StackResources};
use static_cell::StaticCell;

const SOCKET_COUNT: usize = 3;

static RESOURCES: StaticCell<StackResources<SOCKET_COUNT>> = StaticCell::new();

pub fn setup<D>(driver: D) -> (Stack<'static>, Runner<'static, D>)
where
    D: Driver + 'static,
{
    let net_config = Config::dhcpv4(Default::default());
    let resources = RESOURCES.init(StackResources::new());
    let seed = generate_seed();

    embassy_net::new(driver, net_config, resources, seed)
}
