use rcgen::generate_simple_self_signed;
use s2n_quic::{
    Client, Server,
    client::Connect,
    stream::{ReceiveStream, SendStream},
};
use std::{error::Error, net::SocketAddr, path::Path, str::FromStr};
use tokio::{fs::File, io::AsyncReadExt};

pub fn generate_server_certificate(
    username: &str,
) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
    let ck = generate_simple_self_signed(vec![username.to_string()])?;
    Ok((ck.cert.pem(), ck.signing_key.serialize_pem()))
}

pub async fn certificate_and_key_pair_from_files(
    certificate_path: &str,
    key_path: &str,
) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
    let mut certificate_file = File::open(certificate_path).await?;
    let mut key_file = File::open(key_path).await?;

    let mut certificate = String::new();
    certificate_file.read_to_string(&mut certificate).await?;

    let mut key = String::new();
    key_file.read_to_string(&mut key).await?;

    Ok((certificate, key))
}

pub async fn open_connection_stream(port: u32) -> Result<SendStream, Box<dyn Error + Send + Sync>> {
    let mut server = Server::builder()
        .with_tls((Path::new("cert.pem"), Path::new("key.pem")))?
        .with_io(format!("127.0.0.1:{}", port).as_str())?
        .start()?;
    let mut connection = server.accept().await.expect("Should not fail");
    connection.keep_alive(true)?;
    let stream = connection.open_send_stream().await?;
    Ok(stream)
}

pub async fn join_connection_stream(
    server_address: &str,
) -> Result<Option<ReceiveStream>, Box<dyn Error + Send + Sync>> {
    let client = Client::builder()
        .with_tls((Path::new("cert.pem"), Path::new("key.pem")))?
        .with_io("0.0.0.0:0")?
        .start()?;
    let connect = Connect::new(SocketAddr::from_str(server_address)?).with_server_name("localhost");
    let mut connection = client.connect(connect).await?;
    connection.keep_alive(true)?;
    Ok(connection.accept_receive_stream().await?)
}
