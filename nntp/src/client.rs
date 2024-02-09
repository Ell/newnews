use crate::connect::connect;
use crate::error::Error;
use crate::stream::Mode;

pub async fn test() -> Result<(), Error> {
    let socket = connect("127.0.0.1", 8080, Mode::Tls).await?;
    Ok(())
}
