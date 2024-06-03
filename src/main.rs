mod client;
mod database;
mod error;
mod protocol;
mod server;

use std::{
    io::stdin,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    process::exit,
};

use {
    clap::Parser,
    tokio::{select, task::spawn_blocking},
};

use {client::Client, error::Result, protocol::Request, server::Server};

const DEFAULT_SOCKET_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 6530);

#[derive(Debug, Parser)]
#[command(version, about)]
enum Args {
    /// Start a server.
    Server {
        /// The address to bind to.
        #[arg(short, long, default_value_t = DEFAULT_SOCKET_ADDRESS)]
        bind: SocketAddr,
    },
    /// Connect to a server.
    Client {
        /// The address to connect to.
        #[arg(short, long, default_value_t = DEFAULT_SOCKET_ADDRESS)]
        connect: SocketAddr,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(err) = match args {
        Args::Client { connect } => client(connect).await,
        Args::Server { bind } => server(bind).await,
    } {
        eprintln!("{err}");
        exit(1);
    }
}

async fn server(addr: SocketAddr) -> Result {
    let server = Server::bind(addr).await?;

    server.start().await
}

async fn client(addr: SocketAddr) -> Result {
    let mut client = Client::connect(addr).await?;

    loop {
        select! {
            response = client.recv() => {
                println!("{}", response?);
            }
            request = spawn_blocking(read_line) => {
                if let Ok(request) = Request::try_from(request.unwrap().as_str()) {
                    let response = client.request(&request).await?;
                    println!("{response}");
                } else {
                    eprintln!("Cannot parse request");
                }
            }
        }
    }
}

fn read_line() -> String {
    let mut data = String::new();
    stdin().read_line(&mut data).unwrap();
    data
}
