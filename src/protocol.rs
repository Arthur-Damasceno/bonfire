use tokio::{io::AsyncReadExt, net::TcpStream};

#[derive(Debug, Clone)]
pub enum Request {
    Ping,
    Get(Vec<u8>),
    Insert(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

impl Request {
    pub const PING: u8 = 0;
    pub const GET: u8 = 1;
    pub const INSERT: u8 = 2;
    pub const DELETE: u8 = 3;
}

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn accept(&mut self) -> crate::Result<Request> {
        let kind = self.stream.read_u8().await?;

        match kind {
            Request::PING => Ok(Request::Ping),
            Request::GET => {
                let len = self.stream.read_u16().await? as usize;
                let mut key = vec![0; len];

                self.stream.read_exact(&mut key).await?;

                Ok(Request::Get(key))
            }
            Request::INSERT => {
                let key_len = self.stream.read_u16().await? as usize;
                let value_len = self.stream.read_u32().await? as usize;

                let mut key = vec![0; key_len];
                let mut value = vec![0; value_len];

                self.stream.read_exact(&mut key).await?;
                self.stream.read_exact(&mut value).await?;

                Ok(Request::Insert(key, value))
            }
            Request::DELETE => {
                let len = self.stream.read_u16().await? as usize;
                let mut key = vec![0; len];

                self.stream.read_exact(&mut key).await?;

                Ok(Request::Delete(key))
            }
            _ => todo!(),
        }
    }
}
