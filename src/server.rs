use tokio::net::{TcpListener, ToSocketAddrs};

use crate::{
    database::Database,
    protocol::{Connection, Request},
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
                        Request::Ping => println!("Ping!"),
                        Request::Get(key) => {
                            let value = database.get(&key);

                            println!("Get {key:?} = {value:?}");
                        }
                        Request::Insert(key, value) => {
                            println!("Insert {key:?} = {value:?}");

                            database.insert(key, value);
                        }
                        Request::Delete(key) => {
                            if database.delete(&key) {
                                println!("{key:?} deleted");
                            }
                        }
                    }
                }
            });
        }
    }
}
