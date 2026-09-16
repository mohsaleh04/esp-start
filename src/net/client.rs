use core::fmt::Write;
use core::net::Ipv4Addr;
use embassy_net::tcp::{State, TcpSocket};
use embassy_net::{IpAddress, Stack};
use embassy_time::{Duration, Instant, with_timeout};
use esp_hal::Blocking;
use esp_hal::uart::Uart;

use crate::net::socket;

const POLL_TIMEOUT_MS: u64 = 10;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const RESPONSE_BUFFER_SIZE: usize = 512;

pub struct HttpClient {
    socket: TcpSocket<'static>,
    connected: bool,
    server_ip_port: (Ipv4Addr, u16),
    request: &'static [u8],
    reconnect_at: Instant,
    response_buffer: [u8; RESPONSE_BUFFER_SIZE],
}

impl HttpClient {
    pub async fn connect_and_send(
        stack: Stack<'static>,
        server_ip_port: (Ipv4Addr, u16),
        request: &'static [u8],
        uart: &mut Uart<'static, Blocking>,
    ) -> Self {
        let socket = socket::new(stack);
        let mut client = Self {
            socket,
            connected: false,
            server_ip_port,
            request,
            reconnect_at: Instant::now(),
            response_buffer: [0; RESPONSE_BUFFER_SIZE],
        };
        client.try_connect_and_send(uart).await;
        client
    }

    async fn try_connect_and_send(&mut self, uart: &mut Uart<'static, Blocking>) {
        self.socket.abort();
        writeln!(
            uart,
            "[TCP] Connecting to {}:{} ...",
            self.server_ip_port.0, self.server_ip_port.1
        )
        .unwrap();

        self.connected = match with_timeout(
            CONNECT_TIMEOUT,
            socket::connect_tcp(
                &mut self.socket,
                IpAddress::Ipv4(self.server_ip_port.0),
                self.server_ip_port.1,
            ),
        )
        .await
        {
            Ok(Ok(())) if self.socket.state() == State::Established => {
                uart.write_str("[TCP] Connected\r\n").unwrap();
                if let Err(error) = self.socket.write(self.request).await {
                    writeln!(uart, "[TCP] Request failed: {error:?}").unwrap();
                    false
                } else {
                    true
                }
            }
            Ok(Ok(())) => {
                writeln!(uart, "[TCP] Connection failed: {:?}", self.socket.state()).unwrap();
                false
            }
            Ok(Err(error)) => {
                writeln!(uart, "[TCP] Connection failed: {error:?}").unwrap();
                false
            }
            Err(_) => {
                uart.write_str("[TCP] Connection timed out\r\n").unwrap();
                false
            }
        };

        if !self.connected {
            self.reconnect_at = Instant::now() + RECONNECT_DELAY;
        }
    }

    pub async fn poll(&mut self, uart: &mut Uart<'static, Blocking>) {
        if !self.connected {
            if Instant::now() >= self.reconnect_at {
                self.try_connect_and_send(uart).await;
            }
            return;
        }

        match with_timeout(
            Duration::from_millis(POLL_TIMEOUT_MS),
            self.socket.read(&mut self.response_buffer),
        )
        .await
        {
            Ok(Ok(0)) => {
                uart.write_str("[TCP] Connection closed\r\n").unwrap();
                self.connected = false;
                self.reconnect_at = Instant::now() + RECONNECT_DELAY;
            }
            Ok(Ok(size)) => match core::str::from_utf8(&self.response_buffer[..size]) {
                Ok(text) => writeln!(uart, "[TCP] {text}").unwrap(),
                Err(_) => uart.write_str("[TCP] Received non-UTF-8 data\r\n").unwrap(),
            },
            Ok(Err(error)) => {
                writeln!(uart, "[TCP] Read failed: {error:?}").unwrap();
                self.connected = false;
                self.reconnect_at = Instant::now() + RECONNECT_DELAY;
            }
            Err(_) => {}
        }
    }
}
