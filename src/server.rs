use std::sync::Arc;

use tokio::{
    net::{TcpListener, ToSocketAddrs},
    select,
    sync::broadcast::Receiver,
};

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
                let mut channel: Channel = None;

                loop {
                    if channel.is_some() {
                        select! {
                            request = connection.accept() => {
                                if let Ok(request) = request {
                                    Self::handle_request(request, &mut connection, &database, &mut channel).await;
                                } else {
                                    break;
                                }
                            }
                            message = channel.as_mut().unwrap().recv() => {
                                if let Ok(message) = message {
                                    if connection.respond(Response::Message(message)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    } else {
                        if let Ok(request) = connection.accept().await {
                            Self::handle_request(request, &mut connection, &database, &mut channel)
                                .await;
                        } else {
                            break;
                        }
                    }
                }
            });
        }
    }

    async fn handle_request(
        request: Request,
        connection: &mut Connection,
        database: &Database,
        channel: &mut Channel,
    ) {
        let response = match request {
            Request::Ping => Response::Pong,
            Request::Get(key) => {
                if let Some(data) = database.get(&key) {
                    Response::Get(data)
                } else {
                    Response::NotFound
                }
            }
            Request::Set(key, value) => {
                database.set(key, value);

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
                *channel = Some(database.subscribe(id));

                Response::Ok
            }
            Request::Unsubscribe(id) => {
                *channel = None;

                database.unsubscribe(id);

                Response::Ok
            }
        };

        let _ = connection.respond(response).await;
    }
}

type Channel = Option<Receiver<Vec<u8>>>;
