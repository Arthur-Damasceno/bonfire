mod client;
mod database;
mod error;
mod protocol;
mod server;

use std::{env::var, process::exit};

use {client::Client, error::Result, server::Server};

const ADDR: &str = "127.0.0.1:6530";

#[tokio::main]
async fn main() {
    let service = var("SERVICE").unwrap_or_else(|_| "SERVER".into());

    if let Err(err) = match service.as_str() {
        "CLIENT" | "C" => client().await,
        "SERVER" | "S" => server().await,
        _ => panic!("Service {service} does not exists"),
    } {
        eprintln!("{err}");
        exit(1);
    }
}

async fn server() -> Result {
    let server = Server::bind(ADDR).await?;

    server.start().await
}

async fn client() -> Result {
    let _client = Client::connect(ADDR).await?;

    todo!()
}
