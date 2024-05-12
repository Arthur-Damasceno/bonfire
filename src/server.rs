use tokio::net::{TcpListener, ToSocketAddrs};

use crate::{protocol::Connection, database::Database};

pub struct Server {
    listener: TcpListener,
    database: Database,
}

impl Server {
    pub async fn bind(addr: impl ToSocketAddrs) -> crate::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        Ok(Self { listener, database: Database::default() })
    }

    pub async fn start(&self) -> crate::Result {
        loop {
            let (stream, addr) = self.listener.accept().await?;

            println!("Accepted connection from {addr}");

            tokio::spawn(async {
                let mut connection = Connection::new(stream);

                while let Ok(request) = connection.accept().await {
                    println!("{request:?}");
                }
            });
        }
    }
}
