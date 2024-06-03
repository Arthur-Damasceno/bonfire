mod client;
mod database;
mod error;
mod protocol;
mod server;

use std::{env::var, io::stdin, process::exit};

use tokio::{select, task::spawn_blocking};

use {client::Client, error::Result, protocol::Request, server::Server};

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
    let mut client = Client::connect(ADDR).await?;

    loop {
        select! {
            response = client.recv() => {
                println!("{:?}", response?);
            }
            request = spawn_blocking(|| {
                let mut data = String::new();
                stdin().read_line(&mut data).unwrap();
                data
            }) => {
                if let Ok(request) = Request::try_from(request.unwrap().as_str()) {
                    let response = client.request(&request).await?;
                    println!("{response:?}");
                } else {
                    eprintln!("Cannot parse request");
                }
            }
        }
    }
}
