use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug, Clone)]
pub enum Request {
    Ping,
    Get(Vec<u8>),
    Set(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

impl Request {
    pub const PING: u8 = 0;
    pub const GET: u8 = 1;
    pub const SET: u8 = 2;
    pub const DELETE: u8 = 3;
}

#[derive(Debug, Clone)]
pub enum Response {
    Pong,
    Get(Option<Vec<u8>>),
    Set,
    Delete(bool),
}

impl Response {
    pub const PONG: u8 = 0;
    pub const GET: u8 = 1;
    pub const SET: u8 = 2;
    pub const DELETE: u8 = 3;

    pub fn kind(&self) -> u8 {
        match self {
            Self::Pong => Self::PONG,
            Self::Get(_) => Self::GET,
            Self::Set => Self::SET,
            Self::Delete(_) => Self::DELETE,
        }
    }
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
            Request::SET => {
                let key_len = self.stream.read_u16().await? as usize;
                let value_len = self.stream.read_u32().await? as usize;

                let mut key = vec![0; key_len];
                let mut value = vec![0; value_len];

                self.stream.read_exact(&mut key).await?;
                self.stream.read_exact(&mut value).await?;

                Ok(Request::Set(key, value))
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

    pub async fn respond(&mut self, response: Response) -> crate::Result {
        self.stream.write_u8(response.kind()).await?;

        match response {
            Response::Get(ref data) => {
                let len = data.as_ref().map(|data| data.len()).unwrap_or_default();

                self.stream.write_u32(len as u32).await?;

                if let Some(data) = data {
                    self.stream.write_all(&data).await?;
                }
            }
            Response::Delete(deleted) => {
                self.stream.write_u8(deleted as u8).await?;
            }
            _ => {}
        }

        Ok(())
    }
}
