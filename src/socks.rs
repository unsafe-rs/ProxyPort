use std::{io::ErrorKind, net::ToSocketAddrs};

use anyhow::{anyhow, Context, Result};
use fast_socks5::{
    client::{self, Socks5Stream},
    server::{self, Socks5Server, Socks5Socket},
    SocksError,
};
use log::{error, info};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;

pub async fn spawn_socks_service(server_addr: String) -> Result<()> {
    let mut srv = Socks5Server::bind(server_addr).await?;

    let mut config = server::Config::default();
    config.set_transfer_data(false);
    srv.set_config(config);

    let mut incoming = srv.incoming();
    while let Some(res) = incoming.next().await {
        match res {
            Ok(conn) => {
                tokio::spawn(async move {
                    if let Err(err) = handle(conn).await {
                        error!("{:?}", err);
                    }
                });
            }
            Err(e) => {
                error!("accept error: {:?}", e);
            }
        }
    }

    Ok(())
}

// Picking up a backend proxy server.
async fn handle<T>(conn: Socks5Socket<T>) -> Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut local = conn
        .upgrade_to_socks5()
        .await
        .context("upgrade client connection to Socks5")?;

    let sa = local
        .target_addr()
        .context("resolve target address")?
        .to_socket_addrs()?
        .next()
        .context("resolve target of incoming socket")?;

    let mut remote = Socks5Stream::connect(
        "localhost",
        sa.ip().to_string(),
        sa.port(),
        client::Config::default(),
    )
    .await
    .context("connect to upstream proxy")?;

    match tokio::io::copy_bidirectional(&mut local, &mut remote).await {
        Ok(res) => {
            info!("socket transfer closed ({}, {})", res.0, res.1);
            Ok(())
        }
        Err(err) => match err.kind() {
            ErrorKind::NotConnected => {
                info!("socket transfer closed by client");
                Ok(())
            }
            ErrorKind::ConnectionReset => {
                info!("socket transfer closed by downstream proxy");
                Ok(())
            }
            _ => Err(SocksError::Other(anyhow!("socket transfer error: {:#}", err)).into()),
        },
    }
}
