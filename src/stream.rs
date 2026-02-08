use std::error::Error;
use tokio::net::UdpSocket;

pub async fn connect(
    port: u16,
    connection_address: &str,
) -> Result<UdpSocket, Box<dyn Error + Send + Sync>> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
    socket.connect(connection_address).await?;
    Ok(socket)
}
