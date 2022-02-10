use tonic::transport::Server;

mod circuits_routes;

// TODO DRY server creation with the tests
// cf https://github.com/hyperium/tonic/blob/4b0ece6d2854af088fbc1bdb55c2cdd19ec9bb92/tonic-web/tests/integration/tests/grpc.rs#L113
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // TODO configurable port
    let addr = "0.0.0.0:3000".parse().unwrap();
    let ipfs_server_port = 5001;
    let ipfs_server_multiaddr = format!("/ip4/127.0.0.1/tcp/{}", ipfs_server_port);

    let circuits_api = circuits_routes::SkcdApiServerImpl {
        ipfs_server_multiaddr: ipfs_server_multiaddr,
    };
    let circuits_api =
        circuits_routes::interstellarpbapicircuits::skcd_api_server::SkcdApiServer::new(
            circuits_api,
        );
    // let greeter = InterstellarCircuitsApiClient::new(greeter);
    let circuits_api = tonic_web::config()
        .allow_origins(vec!["127.0.0.1"])
        .enable(circuits_api);

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .accept_http1(true)
        .add_service(circuits_api)
        .serve(addr)
        .await?;

    Ok(())
}
