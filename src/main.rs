use clap::Parser;
use tonic::transport::Server;

mod circuits_routes;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// address:port the server will be listening on
    #[clap(long, default_value = "0.0.0.0:3000")]
    bind_addr_port: String,

    /// Where to reach the IPFS node
    #[clap(long, default_value = "/ip4/127.0.0.1/tcp/5001")]
    ipfs_server_multiaddr: String,
}

// TODO DRY server creation with the tests
// cf https://github.com/hyperium/tonic/blob/4b0ece6d2854af088fbc1bdb55c2cdd19ec9bb92/tonic-web/tests/integration/tests/grpc.rs#L113
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let circuits_api = circuits_routes::SkcdApiServerImpl {
        ipfs_server_multiaddr: args.ipfs_server_multiaddr,
    };
    let circuits_api =
        circuits_routes::interstellarpbapicircuits::skcd_api_server::SkcdApiServer::new(
            circuits_api,
        );
    // let greeter = InterstellarCircuitsApiClient::new(greeter);
    let circuits_api = tonic_web::config()
        .allow_origins(vec!["127.0.0.1"])
        .enable(circuits_api);

    let addr = args.bind_addr_port.parse().unwrap();
    println!("Server listening on {}", addr);

    Server::builder()
        .accept_http1(true)
        .add_service(circuits_api)
        .serve(addr)
        .await?;

    Ok(())
}
