use std::sync::Arc;
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    select,
};

use crate::{
    database::Database,
    protocol::{Channel, Connection, Request, Response},
};

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn bind(addr: impl ToSocketAddrs) -> crate::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        Ok(Self { listener })
    }

    pub async fn start(&self) -> crate::Result {
        let database = Arc::new(Database::default());

        loop {
            let (stream, addr) = self.listener.accept().await?;

            println!("Accepted connection from {addr}");

            let database = database.clone();

            tokio::spawn(async move {
                let mut connection = Connection::new(stream);
                let mut channel = Channel::default();

                loop {
                    select! {
                        request = connection.accept() => {
                            if let Ok(request) = request {
                                let _ = Self::handle_request(&mut connection, &database, &mut channel, request).await;
                            } else {
                                break;
                            }
                        }
                        message = channel.recv() => {
                            let _ = connection.respond(Response::Message(message)).await;
                        }
                    }
                }
            });
        }
    }

    async fn handle_request(
        connection: &mut Connection,
        database: &Database,
        channel: &mut Channel,
        request: Request,
    ) -> crate::Result {
        let response = match request {
            Request::Ping => Response::Pong,
            Request::Get(key) => {
                if let Some(data) = database.get(&key) {
                    Response::Get(data)
                } else {
                    Response::NotFound
                }
            }
            Request::Set(key, data) => {
                database.set(key, data);
                Response::Ok
            }
            Request::Delete(key) => {
                if database.delete(&key) {
                    Response::Ok
                } else {
                    Response::NotFound
                }
            }
            Request::Publish(id, message) => {
                if database.publish(id, message) {
                    Response::Ok
                } else {
                    Response::NotFound
                }
            }
            Request::Subscribe(id) => {
                channel.subscribe(database, id);
                Response::Ok
            }
            Request::Unsubscribe => {
                channel.unsubscribe(database);
                Response::Ok
            }
        };

        connection.respond(response).await
    }
}
