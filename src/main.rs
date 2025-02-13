use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::num::NonZeroUsize;

use anyhow::Context;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use tracing::info;

use crate::bench::benchmark;

mod bench;
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
    /// The interface on which to bind the server's listener
    #[arg(long, default_value = "0.0.0.0:55555")]
    addr: SocketAddr,
    /// Serve only grpc (not rest)
    ///
    /// Uses a different server (`tonic::transport::server`) which somehow is
    /// faster, but does not allow regular HTTP routes.
    #[arg(long)]
    grpc_only: bool,
    /// Log parallelism every second for GRPC
    #[arg(long)]
    grpc_logger: bool,
}

#[derive(Debug, Args)]
struct Bench {
    /// The type of workload
    #[arg(long, value_enum)]
    workload: Workload,
    /// How many workers to use (instances of clients, (maybe) number of
    /// connections)
    #[arg(long)]
    workers: NonZeroUsize,
    /// Upper limit of requests per second
    #[arg(long)]
    rate: NonZeroU32,
    /// How many seconds to run the benchmark
    #[arg(long)]
    duration: u64,
    /// Microseconds of jitter for rate limiter
    #[arg(long, default_value = "0")]
    jitter: u64,
    /// Continue benchmarking if error during service call
    ///
    /// Default behavior is to stop at first error
    #[arg(long)]
    continue_on_error: bool,
}

#[derive(Debug, Args)]
struct Client {
    /// Client type
    #[command(subcommand)]
    r#type: ClientType,
    #[command(flatten)]
    bench: Bench,
    /// Where to send requests
    #[arg(long, default_value = "127.0.0.1")]
    hostname: String,
    /// Which port to send requests to
    #[arg(long, default_value = "55555")]
    port: u16,
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

#[derive(Debug, Clone, ValueEnum, Eq, PartialEq, Copy)]
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
        Program::Server(Server {
            addr,
            grpc_only,
            grpc_logger,
        }) => {
            tokio::spawn(async move { server::run(&addr, grpc_only, grpc_logger).await }).await??;
        }
        Program::Client(Client {
            r#type: type_,
            hostname,
            port,
            bench,
        }) => {
            let report = match type_ {
                ClientType::Grpc(_grpc) => {
                    let c = client::grpc::Client::connect(format!("http://{hostname}:{port}"))
                        .await
                        .context("grpc connect")?;
                    benchmark(c, bench).await.context("benchmark")?
                }
                ClientType::Rest(_rest) => {
                    let c = client::rest::Client::new(&format!("http://{hostname}:{port}"));
                    benchmark(c, bench).await.context("benchmark")?
                }
            };
            println!("{report}");
        }
    }

    Ok(())
}
