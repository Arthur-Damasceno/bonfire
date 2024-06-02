use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{
    error::Result,
    protocol::{Request, Response},
};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;

        Ok(Self { stream })
    }

    pub async fn request(&mut self, request: &Request) -> Result<Response> {
        self.stream.write_u8(request.kind()).await?;

        match request {
            Request::Get(key) | Request::Delete(key) => {
                self.stream.write_u16(key.len() as u16).await?;
                self.stream.write_all(&key).await?;
            }
            Request::Set(key, value) => {
                self.stream.write_u16(key.len() as u16).await?;
                self.stream.write_u32(value.len() as u32).await?;
                self.stream.write_all(&key).await?;
                self.stream.write_all(&value).await?;
            }
            Request::Publish(id, data) => {
                self.stream.write_u32(*id).await?;
                self.stream.write_u32(data.len() as u32).await?;
                self.stream.write_all(&data).await?;
            }
            Request::Subscribe(id) => {
                self.stream.write_u32(*id).await?;
            }
            _ => {}
        }

        self.recv().await
    }

    pub async fn recv(&mut self) -> Result<Response> {
        let kind = self.stream.read_u8().await?;

        match kind {
            Response::PONG => Ok(Response::Pong),
            Response::OK => Ok(Response::Ok),
            Response::NOT_FOUND => Ok(Response::NotFound),
            Response::GET => {
                let len = self.stream.read_u32().await? as usize;
                let mut data = vec![0; len];

                self.stream.read_exact(&mut data).await?;

                Ok(Response::Get(data))
            }
            Response::MESSAGE => {
                let len = self.stream.read_u32().await? as usize;
                let mut data = vec![0; len];

                self.stream.read_exact(&mut data).await?;

                Ok(Response::Message(data))
            }
            _ => todo!(),
        }
    }
}
