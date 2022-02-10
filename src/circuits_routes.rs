use ipfs_api_backend_hyper::{Error, TryFromUri};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
// use ipfs_embed::{Config, DefaultParams, Ipfs};
use log;
use std::fs::File;
use std::io::Cursor;
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::Builder;
use tonic::{Request, Response, Status};

use interstellarpbapicircuits::skcd_api_server::SkcdApi;
use interstellarpbapicircuits::skcd_api_server::SkcdApiServer;
use interstellarpbapicircuits::{SkcdDisplayReply, SkcdDisplayRequest};

use lib_circuits_wrapper::cxx::UniquePtr;
use lib_circuits_wrapper::ffi;
use lib_circuits_wrapper::ffi::GenerateDisplaySkcdWrapper;

pub mod interstellarpbapicircuits {
    tonic::include_proto!("interstellarpbapicircuits");
}

// #[derive(Default)]
pub struct SkcdApiServerImpl {
    pub ipfs_server_multiaddr: String,
}

trait HasIpfsClient {
    fn ipfs_client(&self) -> IpfsClient;
}

impl HasIpfsClient for SkcdApiServerImpl {
    fn ipfs_client(&self) -> IpfsClient {
        log::info!(
            "ipfs_client: starting with: {}",
            &self.ipfs_server_multiaddr
        );
        ipfs_api_backend_hyper::IpfsClient::from_multiaddr_str(&self.ipfs_server_multiaddr).unwrap()
    }
}

#[tonic::async_trait]
impl SkcdApi for SkcdApiServerImpl {
    async fn generate_skcd_display(
        &self,
        request: Request<SkcdDisplayRequest>,
    ) -> Result<Response<SkcdDisplayReply>, Status> {
        log::info!("Got a request from {:?}", request.remote_addr());
        let width = request.get_ref().width;
        let height = request.get_ref().height;

        // TODO class member/Trait for "lib_circuits_wrapper::ffi::new_circuit_gen_wrapper()"
        let lib_circuits_wrapper = tokio::task::spawn_blocking(move || {
            let tmp_dir = Builder::new()
                .prefix("interstellar-circuit_routes")
                .tempdir()
                .unwrap();

            let file_path = tmp_dir.path().join("output.skcd.pb.bin");

            let wrapper = lib_circuits_wrapper::ffi::new_circuit_gen_wrapper();

            // TODO make the C++ API return a buffer?
            wrapper.GenerateDisplaySkcd(file_path.as_os_str().to_str().unwrap(), width, height);

            let contents = std::fs::read(file_path).expect("Something went wrong reading the file");

            contents
        })
        .await
        .unwrap();

        let data = Cursor::new(lib_circuits_wrapper);

        // TODO error handling, or at least logging
        let ipfs_result = self.ipfs_client().add(data).await.unwrap();

        let reply = SkcdDisplayReply {
            hash: format!("{}", ipfs_result.hash),
        };

        Ok(Response::new(reply))
    }
}
