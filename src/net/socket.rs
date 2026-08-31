use embassy_net::tcp::TcpSocket;
use embassy_net::{IpAddress, IpEndpoint, Stack};
use static_cell::StaticCell;

static RX_BUFF: StaticCell<[u8; 1024]> = StaticCell::new();
static TX_BUFF: StaticCell<[u8; 1024]> = StaticCell::new();

pub fn new(stack: Stack<'static>) -> TcpSocket<'static> {
    let rx_buff = RX_BUFF.init([0u8; 1024]);
    let tx_buff = TX_BUFF.init([0u8; 1024]);

    TcpSocket::new(stack, rx_buff, tx_buff)
}

pub async fn connect_tcp(socket: &mut TcpSocket<'static>, server_ip: IpAddress, server_port: u16) {
    let server_endpoint = IpEndpoint::new(server_ip, server_port);
    socket
        .connect(server_endpoint)
        .await
        .expect("Didn't expect connection TCP failed")
}
