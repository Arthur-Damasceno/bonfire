use std::sync::Arc;

use tokio::net::{TcpListener, ToSocketAddrs};

use crate::{
    database::Database,
    protocol::{Connection, Request, Response},
};

pub struct Server {
    listener: TcpListener,
    database: Arc<Database>,
}

impl Server {
    pub async fn bind(addr: impl ToSocketAddrs) -> crate::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        Ok(Self {
            listener,
            database: Default::default(),
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
                            let response = {
                                if let Some(data) = database.get(&key) {
                                    Response::Get(data)
                                } else {
                                    Response::NotFound
                                }
                            };

                            let _ = connection.respond(response).await;
                        }
                        Request::Set(key, value) => {
                            database.set(key, value);

                            let _ = connection.respond(Response::Ok).await;
                        }
                        Request::Delete(key) => {
                                let response = if database.delete(&key) {Response::Ok} else {Response::NotFound};

                            let _ = connection.respond(response).await;
                        }
                        _ => todo!()
                    }
                }
            });
        }
    }
}
