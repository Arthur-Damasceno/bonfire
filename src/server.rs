use tokio::net::{TcpListener, ToSocketAddrs};

use crate::{
    database::Database,
    protocol::{Connection, Request, Response},
};

pub struct Server {
    listener: TcpListener,
    database: Database,
}

impl Server {
    pub async fn bind(addr: impl ToSocketAddrs) -> crate::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        Ok(Self {
            listener,
            database: Database::default(),
        })
    }

    pub async fn start(&self) -> crate::Result {
        loop {
            let (stream, addr) = self.listener.accept().await?;

            println!("Accepted connection from {addr}");

            let database = self.database.clone();

            tokio::spawn(async move {
                let mut connection = Connection::new(stream);

                while let Ok(request) = connection.accept().await {
                    match request {
                        Request::Ping => {
                            let _ = connection.respond(Response::Pong).await;             
                        }
                        Request::Get(key) => {
                            let value = database.get(&key);

                            let _ = connection.respond(Response::Get(value)).await;
                        }
                        Request::Set(key, value) => {
                            database.set(key, value);

                            let _ = connection.respond(Response::Set).await;
                        }
                        Request::Delete(key) => {
                            let deleted = database.delete(&key);

                            let _ = connection.respond(Response::Delete(deleted)).await;
                        }
                    }
                }
            });
        }
    }
}
