mod client;
pub mod runner;
pub mod socket;

pub use client::HttpClient;

use crate::utils::generate_seed;
use crate::wifi;
use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_net::driver::Driver;
use embassy_net::{Config, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::Blocking;
use esp_hal::peripherals::WIFI;
use esp_hal::uart::Uart;
use esp_radio::wifi::WifiController;
use static_cell::StaticCell;

const SOCKET_COUNT: usize = 3;

static RESOURCES: StaticCell<StackResources<SOCKET_COUNT>> = StaticCell::new();

pub struct WifiNetwork {
    stack: Stack<'static>,
    _controller: WifiController<'static>,
}

impl WifiNetwork {
    pub async fn connect(
        wifi_peripheral: WIFI<'static>,
        spawner: Spawner,
        ssid: &str,
        password: Option<&str>,
        uart: &mut Uart<'static, Blocking>,
    ) -> Option<Self> {
        uart.write_str("[WIFI] Initializing ... ").unwrap();
        let (mut controller, station) = match wifi::setup(wifi_peripheral) {
            Ok(wifi) => wifi,
            Err(error) => {
                writeln!(uart, "FAILED: {error:?}").unwrap();
                return None;
            }
        };
        if let Err(error) = wifi::config::set_station_config(&mut controller, ssid, password) {
            writeln!(uart, "FAILED: invalid station config: {error:?}").unwrap();
            return None;
        }

        let (stack, runner) = setup(station);
        runner::run_wifi_net_task(spawner, runner);
        uart.write_str("SUCCESS\r\n[WIFI] Connecting ...\r\n")
            .unwrap();

        if !wifi::connection::connect(&mut controller, uart).await {
            return None;
        }

        wait_for_config_up(stack, uart).await;
        uart.write_str("[NET] Network is up\r\n").unwrap();
        if let Some(config) = stack.config_v4() {
            writeln!(uart, "[NET] IPv4 config: {config:?}").unwrap();
        }

        Some(Self {
            stack,
            _controller: controller,
        })
    }

    pub fn stack(&self) -> Stack<'static> {
        self.stack
    }
}

pub fn setup<D>(driver: D) -> (Stack<'static>, Runner<'static, D>)
where
    D: Driver + 'static,
{
    let net_config = Config::dhcpv4(Default::default());
    let resources = RESOURCES.init(StackResources::new());
    let seed = generate_seed();

    embassy_net::new(driver, net_config, resources, seed)
}

async fn wait_for_config_up(stack: Stack<'static>, uart: &mut Uart<'static, Blocking>) {
    while !stack.is_config_up() {
        uart.write_str("[NET] Waiting for DHCP ...\r\n").unwrap();
        Timer::after(Duration::from_millis(500)).await;
    }
}
