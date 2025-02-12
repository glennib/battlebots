use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::num::NonZeroUsize;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use tracing::info;

mod client;
mod proto;
mod server;
mod workloads;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    program: Program,
}

#[derive(Debug, Subcommand)]
enum Program {
    /// Run as server, serving both gRPC and HTTP/REST requests
    Server(Server),
    /// Run as either a gRPC client or an HTTP/REST client
    Client(Client),
}

#[derive(Debug, Args)]
struct Server {
    #[arg(default_value = "0.0.0.0:55555")]
    addr: SocketAddr,
}

#[derive(Debug, Args)]
struct Client {
    #[command(subcommand)]
    r#type: ClientType,
    /// The type of workload
    #[arg(value_enum)]
    workload: Workload,
    workers: NonZeroUsize,
    rate: NonZeroU32,
    duration: u64,
    jitter: u64,
    hostname: String,
    port: Option<u16>,
}

#[derive(Debug, Subcommand)]
enum ClientType {
    /// Run as gRPC client
    Grpc(Grpc),
    /// Run as HTTP/REST client
    Rest(Rest),
}

#[derive(Debug, Args)]
struct Grpc {}

#[derive(Debug, Args)]
struct Rest {}

#[derive(Debug, Clone, ValueEnum)]
enum Workload {
    Inty,
    Stringy,
    Mixed,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    info!(?cli);

    match cli.program {
        Program::Server(Server { addr }) => {
            server::run(&addr).await?;
        }
        Program::Client(_) => {}
    }

    Ok(())
}
