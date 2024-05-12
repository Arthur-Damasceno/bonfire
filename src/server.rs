use tokio::net::{TcpListener, ToSocketAddrs};

#[derive(Debug)]
pub struct Server {
    listener: TcpListener
}

impl Server {
    pub async fn bind(addr: impl ToSocketAddrs) -> crate::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        Ok(Self { listener })
    }

    pub async fn accept(&self) -> crate::Result {
        let (_, addr) = self.listener.accept().await?;

        println!("Accepted request from {addr}");

        Ok(())
    }
}
