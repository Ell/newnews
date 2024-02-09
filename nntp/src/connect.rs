use crate::error::Error;
use crate::stream::{MaybeTlsStream, Mode};
use crate::tls::client_async_tls;

use tokio::net::TcpStream;
use tokio_codec::{Decoder, LinesCodec};

pub async fn connect(
    host: &str,
    port: u16,
    mode: Mode,
) -> Result<MaybeTlsStream<TcpStream>, Error> {
    let address = format!("{host}:{port}");
    let socket = TcpStream::connect(address).await.map_err(Error::Io)?;

    let codec = LinesCodec::new();
    // let (sink, input) = codec.framed(socke)

    let connection = client_async_tls(socket, host.to_string(), mode, None).await?;
    let mut test = codec.framed(connection);

    todo!()
}
