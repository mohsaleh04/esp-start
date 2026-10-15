use crate::net::{error::HttpClientError, request::HttpRequest, socket};
use core::fmt::Write;
use core::net::Ipv4Addr;
use embassy_net::tcp::{State, TcpSocket};
use embassy_net::{IpAddress, Stack};
use embassy_time::{Duration, with_timeout};
use heapless::String;
use reqwless::client::HttpConnection;
use reqwless::request::{Request, RequestBuilder};
use reqwless::response::StatusCode;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const RESPONSE_BUFFER_SIZE: usize = 4096;

pub struct HttpClient {
    socket: TcpSocket<'static>,
    server_ip_port: (Ipv4Addr, u16),
    response_buffer: [u8; RESPONSE_BUFFER_SIZE],
}

impl HttpClient {
    pub fn new(stack: Stack<'static>, server_ip_port: (Ipv4Addr, u16)) -> Self {
        Self {
            socket: socket::new(stack),
            server_ip_port,
            response_buffer: [0; RESPONSE_BUFFER_SIZE],
        }
    }

    pub async fn send<F, R>(
        &mut self,
        request: HttpRequest<'_>,
        on_response: F,
    ) -> Result<R, HttpClientError>
    where
        F: FnOnce(StatusCode, &[u8]) -> R,
    {
        self.socket.abort();

        let connection_result = with_timeout(
            CONNECT_TIMEOUT,
            socket::connect_tcp(
                &mut self.socket,
                IpAddress::Ipv4(self.server_ip_port.0),
                self.server_ip_port.1,
            ),
        )
        .await;
        match connection_result {
            Ok(Ok(())) if self.socket.state() == State::Established => {}
            Err(_) => {
                self.socket.abort();
                return Err(HttpClientError::ConnectTimeout);
            }
            _ => {
                self.socket.abort();
                return Err(HttpClientError::ConnectFailed);
            }
        }

        let connection_host = self.print_host()?;
        let mut connection = HttpConnection::Plain(&mut self.socket);

        let request_builder = Request::new(request.method, request.path)
            .host(connection_host.as_str())
            .headers(request.headers);
        let response_result = match request.body {
            Some(body) => {
                connection
                    .send(
                        request_builder.body(body).build(),
                        &mut self.response_buffer,
                    )
                    .await
            }
            None => {
                connection
                    .send(request_builder.build(), &mut self.response_buffer)
                    .await
            }
        };

        let response = match response_result {
            Ok(response) => response,
            Err(_) => {
                self.socket.abort();
                return Err(HttpClientError::RequestFailed);
            }
        };
        let status = response.status;
        let body = match response.body().read_to_end().await {
            Ok(body) => body,
            Err(_) => {
                self.socket.abort();
                return Err(HttpClientError::BodyReadFailed);
            }
        };

        let result = on_response(status, body);
        self.socket.abort();
        Ok(result)
    }

    fn print_host(&self) -> Result<String<32>, HttpClientError> {
        let mut host: String<32> = String::new();
        write!(host, "{}:{}", self.server_ip_port.0, self.server_ip_port.1)
            .map_err(move |_| HttpClientError::RequestFailed)?;
        Ok(host)
    }
}
