use anyhow::Result;
use tokio::net::TcpListener;

pub async fn spawn_web_service(listen_on: String) -> Result<()> {
    let lis = TcpListener::bind(listen_on).await?;
    loop {
        let (stream, _) = lis.accept().await?;
    }
}
