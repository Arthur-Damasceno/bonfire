mod database;
mod error;
mod protocol;
mod server;

use std::process::exit;

use error::Result;
use server::Server;

#[tokio::main]
async fn main() -> Result {
    let server = Server::bind("127.0.0.1:6530").await?;

    if let Err(err) = server.start().await {
        eprint!("{err}");
        exit(1);
    }

    unreachable!()
}
