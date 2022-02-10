// https://github.com/hyperium/tonic/issues/727
// https://github.com/hyperium/tonic/blob/master/tests/integration_tests/tests/timeout.rs

// TODO? use integration_tests::pb::{test_client, test_server, Input, Output};
// use ipfs_embed::{Config, DefaultParams, Ipfs};
use bytes::Buf;
use bytes::BufMut;
use prost::Message;
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tonic::{transport::Server, Code, Request, Response, Status};

// mod circuits_routes;
use api_circuits::circuits_routes::{self, interstellarpbapicircuits::SkcdDisplayReply};

mod foreign_ipfs;

pub mod interstellarpbapicircuits {
    tonic::include_proto!("interstellarpbapicircuits");
}

#[tokio::test]
async fn endpoint_generate_circuit_protobuf() {
    let foreign_node = run_ipfs_in_background().await;
    let ipfs_server_multiaddr = format!("/ip4/127.0.0.1/tcp/{}", foreign_node.api_port);
    let addr = run_service_in_background(
        Duration::from_secs(1),
        Duration::from_secs(100),
        ipfs_server_multiaddr,
    )
    .await;

    let mut client = interstellarpbapicircuits::skcd_api_client::SkcdApiClient::connect(format!(
        "http://{}",
        addr
    ))
    .await
    .unwrap();

    let mut req = Request::new(interstellarpbapicircuits::SkcdDisplayRequest {
        width: 224,
        height: 96,
    });
    req.metadata_mut()
        // TODO less than 5000 ms!
        .insert("grpc-timeout", "5000m".parse().unwrap());

    let res = client.generate_skcd_display(req).await;

    let resp = res.unwrap();
    // assert!(ok.message.contains("OK"));
    // TODO better check
    assert_eq!(
        resp.get_ref().hash.len(),
        "Qmf1rtki74jvYmGeqaaV51hzeiaa6DyWc98fzDiuPatzyy".len()
    );
}

// we CAN NOT just send the raw encoded protobuf(eg using SkcdDisplayRequest{}.encode())
// b/c that returns errors like
// "protocol error: received message with invalid compression flag: 8 (valid flags are 0 and 1), while sending request"
// "tonic-web: Invalid byte 45, offset 0"
// https://github.com/hyperium/tonic/blob/01e5be508051eebf19c233d48b57797a17331383/tonic-web/tests/integration/tests/grpc_web.rs#L93
// also: https://github.com/grpc/grpc-web/issues/152
fn encode_body(width: u32, height: u32) -> bytes::Bytes {
    let input = interstellarpbapicircuits::SkcdDisplayRequest {
        width: width,
        height: height,
    };

    let mut buf = bytes::BytesMut::with_capacity(1024);
    buf.reserve(5);
    unsafe {
        buf.advance_mut(5);
    }

    input.encode(&mut buf).unwrap();

    let len = buf.len() - 5;
    {
        let mut buf = &mut buf[..5];
        buf.put_u8(0);
        buf.put_u32(len as u32);
    }

    buf.split_to(len + 5).freeze()
}

async fn decode_body(body: hyper::Body, content_type: &str) -> (SkcdDisplayReply, bytes::Bytes) {
    let mut body = hyper::body::to_bytes(body).await.unwrap();

    if content_type == "application/grpc-web-text+proto" {
        body = base64::decode(body).unwrap().into()
    }

    body.advance(1);
    let len = body.get_u32();
    let msg = SkcdDisplayReply::decode(&mut body.split_to(len as usize)).expect("decode");
    body.advance(5);

    (msg, body)
}

// TODO WARNING verilog code is NOT thread safe! MUST be run test by test
//  cargo test -- --test-threads=1
#[tokio::test]
async fn endpoint_generate_circuit_grpc_web() {
    let foreign_node = run_ipfs_in_background().await;
    let ipfs_server_multiaddr = format!("/ip4/127.0.0.1/tcp/{}", foreign_node.api_port);
    let addr = run_service_in_background(
        Duration::from_secs(1),
        Duration::from_secs(100),
        ipfs_server_multiaddr,
    )
    .await;

    let request_uri = format!(
        "http://{}/interstellarpbapicircuits.SkcdApi/GenerateSkcdDisplay",
        addr
    );

    let client = hyper::Client::new();

    let body_buf = encode_body(224, 96);

    let content_type = "application/grpc-web";
    let accept = "application/grpc-web";
    let req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header(hyper::header::CONTENT_TYPE, content_type)
        // .header(hyper::header::ORIGIN, "http://example.com")
        .header(hyper::header::ACCEPT, accept)
        .uri(request_uri)
        .body(hyper::Body::from(body_buf))
        .unwrap();

    let res = client.request(req).await.unwrap();

    assert_eq!(res.status(), hyper::StatusCode::OK);
    let (reply, trailers) = decode_body(res.into_body(), content_type).await;
    assert_eq!(
        reply.hash.len(),
        "Qmf1rtki74jvYmGeqaaV51hzeiaa6DyWc98fzDiuPatzyy".len()
    );
    assert_eq!(&trailers[..], b"grpc-status:0\r\n");
}

// #[tokio::test]
// async fn picks_server_timeout_if_thats_sorter() {
//     let addr = run_service_in_background(Duration::from_secs(1), Duration::from_millis(100)).await;

//     let mut client = test_client::TestClient::connect(format!("http://{}", addr))
//         .await
//         .unwrap();

//     let mut req = Request::new(Input {});
//     req.metadata_mut()
//         // 10 hours
//         .insert("grpc-timeout", "10H".parse().unwrap());

//     let res = client.unary_call(req).await;
//     let err = res.unwrap_err();
//     assert!(err.message().contains("Timeout expired"));
//     assert_eq!(err.code(), Code::Cancelled);
// }

// #[tokio::test]
// async fn picks_client_timeout_if_thats_sorter() {
//     let addr = run_service_in_background(Duration::from_secs(1), Duration::from_secs(100)).await;

//     let mut client = test_client::TestClient::connect(format!("http://{}", addr))
//         .await
//         .unwrap();

//     let mut req = Request::new(Input {});
//     req.metadata_mut()
//         // 100 ms
//         .insert("grpc-timeout", "100m".parse().unwrap());

//     let res = client.unary_call(req).await;
//     let err = res.unwrap_err();
//     assert!(err.message().contains("Timeout expired"));
//     assert_eq!(err.code(), Code::Cancelled);
// }

async fn run_service_in_background(
    latency: Duration,
    server_timeout: Duration,
    ipfs_server_multiaddr: String,
) -> SocketAddr {
    // struct Svc {
    //     latency: Duration,
    // }

    // #[tonic::async_trait]
    // impl interstellarpbapicircuits::circuits_api_server::CircuitsApi for Svc {
    //     async fn unary_call(&self, _req: Request<Input>) -> Result<Response<Output>, Status> {
    //         tokio::time::sleep(self.latency).await;
    //         Ok(Response::new(Output {}))
    //     }
    // }

    // let svc =
    //     interstellarpbapicircuits::circuits_api_server::CircuitsApiServer::new(Svc { latency });

    // let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    // let addr = listener.local_addr().unwrap();

    // tokio::spawn(async move {
    //     Server::builder()
    //         .timeout(server_timeout)
    //         .add_service(svc)
    //         .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
    //         .await
    //         .unwrap();
    // });

    // tokio::spawn(async move {
    //     Server::builder()
    //         .timeout(server_timeout)
    //         .add_service(svc)
    //         .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
    //         .await
    //         .unwrap();
    // });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

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

    tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(circuits_api)
            // .serve(addr) // NO!
            // thread 'cancelation_on_timeout' panicked at 'called `Result::unwrap()` on an `Err`
            // value: tonic::transport::Error(Transport, hyper::Error(Connect, ConnectError("tcp connect error",
            // Os { code: 111, kind: ConnectionRefused, message: "Connection refused" })))',
            // tests/circuit_gen_endpoint_test.rs:24:6
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    addr
}

// https://github.com/ipfs-rust/ipfs-embed/#getting-started
async fn run_ipfs_in_background() -> foreign_ipfs::ForeignNode {
    // let cache_size = 10;
    // let ipfs = Ipfs::<DefaultParams>::new(Config::default()).await.unwrap();
    // ipfs.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap());

    // tokio::spawn(async move {

    // });

    // https://github.com/rs-ipfs/rust-ipfs/blob/master/tests/pubsub.rs
    let foreign_node = foreign_ipfs::ForeignNode::new();
    let foreign_api_port = foreign_node.api_port;
    println!("run_ipfs_in_background: port: {}", foreign_api_port);

    // MUST be returned and kept alive; else the daemon is killed
    foreign_node

    // ALTERNATIVE: https://docs.ipfs.io/install/ipfs-desktop/#ubuntu
}
