use core::fmt::Write;
use core::net::Ipv4Addr;
use embassy_net::tcp::{State, TcpSocket};
use embassy_net::{IpAddress, Stack};
use embassy_time::{Duration, with_timeout};
use esp_hal::Blocking;
use esp_hal::uart::Uart;

use crate::net::socket;

const POLL_TIMEOUT_MS: u64 = 10;
const RESPONSE_BUFFER_SIZE: usize = 512;

pub struct HttpClient {
    socket: TcpSocket<'static>,
    connected: bool,
    response_buffer: [u8; RESPONSE_BUFFER_SIZE],
}

impl HttpClient {
    pub async fn connect_and_send(
        stack: Stack<'static>,
        server_ip: Ipv4Addr,
        server_port: u16,
        request: &[u8],
        uart: &mut Uart<'static, Blocking>,
    ) -> Self {
        let mut socket = socket::new(stack);
        writeln!(uart, "[TCP] Connecting to {server_ip}:{server_port} ...").unwrap();

        let connected =
            match socket::connect_tcp(&mut socket, IpAddress::Ipv4(server_ip), server_port).await {
                Ok(()) if socket.state() == State::Established => {
                    uart.write_str("[TCP] Connected\r\n").unwrap();
                    if let Err(error) = socket.write(request).await {
                        writeln!(uart, "[TCP] Request failed: {error:?}").unwrap();
                        false
                    } else {
                        true
                    }
                }
                Ok(()) => {
                    writeln!(uart, "[TCP] Connection failed: {:?}", socket.state()).unwrap();
                    false
                }
                Err(error) => {
                    writeln!(uart, "[TCP] Connection failed: {error:?}").unwrap();
                    false
                }
            };

        Self {
            socket,
            connected,
            response_buffer: [0; RESPONSE_BUFFER_SIZE],
        }
    }

    pub async fn poll(&mut self, uart: &mut Uart<'static, Blocking>) {
        if !self.connected {
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
            }
            Ok(Ok(size)) => match core::str::from_utf8(&self.response_buffer[..size]) {
                Ok(text) => writeln!(uart, "[TCP] {text}").unwrap(),
                Err(_) => uart.write_str("[TCP] Received non-UTF-8 data\r\n").unwrap(),
            },
            Ok(Err(error)) => {
                writeln!(uart, "[TCP] Read failed: {error:?}").unwrap();
                self.connected = false;
            }
            Err(_) => {}
        }
    }
}
