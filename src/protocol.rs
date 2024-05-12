use tokio::{io::AsyncReadExt, net::TcpStream};

#[derive(Debug, Clone)]
pub enum Request {
    Ping,
}

impl Request {
    pub const PING: u8 = 0;
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
            _ => todo!(),
        }
    }
}
