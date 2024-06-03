use std::time::Duration;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::Receiver,
    time::sleep,
};

use crate::{
    database::Database,
    error::{Error, Result},
};

#[derive(Debug, Clone)]
pub enum Request {
    Ping,
    Get(Vec<u8>),
    Set(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
    Publish(u32, Vec<u8>),
    Subscribe(u32),
    Unsubscribe,
}

impl Request {
    pub const PING: u8 = 0;
    pub const GET: u8 = 1;
    pub const SET: u8 = 2;
    pub const DELETE: u8 = 3;
    pub const PUBLISH: u8 = 4;
    pub const SUBSCRIBE: u8 = 5;
    pub const UNSUBSCRIBE: u8 = 6;

    pub fn kind(&self) -> u8 {
        match self {
            Self::Ping => Self::PING,
            Self::Get(_) => Self::GET,
            Self::Set(_, _) => Self::SET,
            Self::Delete(_) => Self::DELETE,
            Self::Publish(_, _) => Self::PUBLISH,
            Self::Subscribe(_) => Self::SUBSCRIBE,
            Self::Unsubscribe => Self::UNSUBSCRIBE,
        }
    }
}

impl TryFrom<&str> for Request {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut args = value.split_whitespace();

        let kind = args.next().ok_or(Error::Parse).and_then(|kind| {
            match kind.to_ascii_uppercase().as_str() {
                "PING" | "P" => Ok(Self::PING),
                "GET" | "G" => Ok(Self::GET),
                "SET" | "S" => Ok(Self::SET),
                "DELETE" | "D" => Ok(Self::DELETE),
                "PUBLISH" | "PUB" => Ok(Self::PUBLISH),
                "SUBSCRIBE" | "SUB" => Ok(Self::SUBSCRIBE),
                "UNSUBSCRIBE" | "UNSUB" => Ok(Self::UNSUBSCRIBE),
                _ => Err(Error::Parse),
            }
        })?;

        Ok(match kind {
            Self::PING => Self::Ping,
            Self::GET => {
                let key = args.next().ok_or(Error::Parse)?.as_bytes().to_vec();
                Self::Get(key)
            }
            Self::SET => {
                let key = args.next().ok_or(Error::Parse)?.as_bytes().to_vec();
                let value = args.next().ok_or(Error::Parse)?.as_bytes().to_vec();
                Self::Set(key, value)
            }
            Self::DELETE => {
                let key = args.next().ok_or(Error::Parse)?.as_bytes().to_vec();
                Self::Delete(key)
            }
            Self::PUBLISH => {
                let id = args.next().ok_or(Error::Parse).map(|id| id.parse())??;
                let data = args.next().ok_or(Error::Parse)?.as_bytes().to_vec();
                Self::Publish(id, data)
            }
            Self::SUBSCRIBE => {
                let id = args.next().ok_or(Error::Parse).map(|id| id.parse())??;
                Self::Subscribe(id)
            }
            Self::UNSUBSCRIBE => Self::Unsubscribe,
            _ => unreachable!(),
        })
    }
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

pub struct Connection(TcpStream);

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    pub async fn accept(&mut self) -> Result<Request> {
        let kind = self.0.read_u8().await?;

        match kind {
            Request::PING => Ok(Request::Ping),
            Request::GET => {
                let len = self.0.read_u16().await? as usize;
                let mut key = vec![0; len];

                self.0.read_exact(&mut key).await?;

                Ok(Request::Get(key))
            }
            Request::SET => {
                let key_len = self.0.read_u16().await? as usize;
                let value_len = self.0.read_u32().await? as usize;

                let mut key = vec![0; key_len];
                let mut value = vec![0; value_len];

                self.0.read_exact(&mut key).await?;
                self.0.read_exact(&mut value).await?;

                Ok(Request::Set(key, value))
            }
            Request::DELETE => {
                let len = self.0.read_u16().await? as usize;
                let mut key = vec![0; len];

                self.0.read_exact(&mut key).await?;

                Ok(Request::Delete(key))
            }
            Request::PUBLISH => {
                let id = self.0.read_u32().await?;
                let len = self.0.read_u32().await? as usize;
                let mut message = vec![0; len];

                self.0.read_exact(&mut message).await?;

                Ok(Request::Publish(id, message))
            }
            Request::SUBSCRIBE => {
                let id = self.0.read_u32().await?;

                Ok(Request::Subscribe(id))
            }
            Request::UNSUBSCRIBE => Ok(Request::Unsubscribe),
            _ => todo!(),
        }
    }

    pub async fn respond(&mut self, response: Response) -> Result {
        self.0.write_u8(response.kind()).await?;

        if let Response::Get(data) | Response::Message(data) = response {
            self.0.write_u32(data.len() as u32).await?;
            self.0.write_all(&data).await?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Channel(Option<(u32, Receiver<Vec<u8>>)>);

impl Channel {
    pub fn subscribe(&mut self, database: &Database, id: u32) {
        if let Some((id, _)) = self.0 {
            self.0 = None;
            database.unsubscribe(id);
        }

        let rx = database.subscribe(id);
        self.0 = Some((id, rx));
    }

    pub fn unsubscribe(&mut self, database: &Database) {
        if let Some((id, _)) = self.0 {
            self.0 = None;
            database.unsubscribe(id);
        }
    }

    pub async fn recv(&mut self) -> Vec<u8> {
        if let Some((_, rx)) = &mut self.0 {
            rx.recv().await.unwrap()
        } else {
            sleep(Duration::from_secs(u64::MAX)).await;
            unreachable!()
        }
    }
}
