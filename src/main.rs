mod database;
mod protocol;
mod server;

use server::Server;

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result {
    let server = Server::bind("127.0.0.1:6530").await?;

    server.start().await
}
