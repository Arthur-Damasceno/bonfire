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
    Publish(u32, Vec<u8>),
    Subscribe(u32),
    Unsubscribe(u32),
}

impl Request {
    pub const PING: u8 = 0;
    pub const GET: u8 = 1;
    pub const SET: u8 = 2;
    pub const DELETE: u8 = 3;
    pub const PUBLISH: u8 = 4;
    pub const SUBSCRIBE: u8 = 5;
    pub const UNSUBSCRIBE: u8 = 6;
}

#[derive(Debug, Clone)]
pub enum Response {
    Pong,
    Ok,
    NotFound,
    Get(Vec<u8>),
    Message(Vec<u8>),
}

impl Response {
    pub const PONG: u8 = 0;
    pub const OK: u8 = 1;
    pub const NOT_FOUND: u8 = 2;
    pub const GET: u8 = 3;
    pub const MESSAGE: u8 = 4;

    pub fn kind(&self) -> u8 {
        match self {
            Self::Pong => Self::PONG,
            Self::Ok => Self::OK,
            Self::NotFound => Self::NOT_FOUND,
            Self::Get(_) => Self::GET,
            Self::Message(_) => Self::MESSAGE,
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
            Request::PUBLISH => {
                let id = self.stream.read_u32().await?;
                let len = self.stream.read_u32().await? as usize;
                let mut message = vec![0; len];

                self.stream.read_exact(&mut message).await?;

                Ok(Request::Publish(id, message))
            }
            Request::SUBSCRIBE => {
                let id = self.stream.read_u32().await?;

                Ok(Request::Subscribe(id))
            }
            Request::UNSUBSCRIBE => {
                let id = self.stream.read_u32().await?;

                Ok(Request::Unsubscribe(id))
            }
            _ => todo!(),
        }
    }

    pub async fn respond(&mut self, response: Response) -> crate::Result {
        self.stream.write_u8(response.kind()).await?;

        if let Response::Get(data) | Response::Message(data) = response {
            self.stream.write_u32(data.len() as u32).await?;
            self.stream.write_all(&data).await?;
        }

        Ok(())
    }
}
