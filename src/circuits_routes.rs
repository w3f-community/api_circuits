// Copyright 2022 Nathan Prat

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use futures_util::TryStreamExt;
use ipfs_api_backend_hyper::{
    BackendWithGlobalOptions, GlobalOptions, IpfsApi, IpfsClient, TryFromUri,
};
use std::io::Cursor;
use std::time::Duration;
use tempfile::Builder;
use tonic::{Request, Response, Status};

use interstellarpbapicircuits::skcd_api_server::SkcdApi;
pub use interstellarpbapicircuits::skcd_api_server::SkcdApiServer;
use interstellarpbapicircuits::{
    SkcdDisplayReply, SkcdDisplayRequest, SkcdGenericFromIpfsReply, SkcdGenericFromIpfsRequest,
};

pub mod interstellarpbapicircuits {
    tonic::include_proto!("interstellarpbapicircuits");
}

// #[derive(Default)]
pub struct SkcdApiServerImpl {
    pub ipfs_server_multiaddr: String,
}

trait HasIpfsClient {
    fn ipfs_client(&self) -> BackendWithGlobalOptions<IpfsClient>;
}

impl HasIpfsClient for SkcdApiServerImpl {
    fn ipfs_client(&self) -> BackendWithGlobalOptions<IpfsClient> {
        log::info!(
            "ipfs_client: starting with: {}",
            &self.ipfs_server_multiaddr
        );
        BackendWithGlobalOptions::new(
            ipfs_api_backend_hyper::IpfsClient::from_multiaddr_str(&self.ipfs_server_multiaddr)
                .unwrap(),
            GlobalOptions::builder()
                .timeout(Duration::from_millis(5000))
                .build(),
        )
    }
}

#[tonic::async_trait]
impl SkcdApi for SkcdApiServerImpl {
    async fn generate_skcd_display(
        &self,
        request: Request<SkcdDisplayRequest>,
    ) -> Result<Response<SkcdDisplayReply>, Status> {
        log::info!(
            "generate_skcd_display request from {:?}",
            request.remote_addr()
        );
        let width = request.get_ref().width;
        let height = request.get_ref().height;
        let digits_bboxes = request.get_ref().digits_bboxes.clone();

        // TODO class member/Trait for "lib_circuits_wrapper::ffi::new_circuit_gen_wrapper()"
        let lib_circuits_wrapper = tokio::task::spawn_blocking(move || {
            let wrapper = lib_circuits_wrapper::ffi::new_circuit_gen_wrapper();

            wrapper.GenerateDisplaySkcd(width, height, &digits_bboxes)
        })
        .await
        .unwrap();

        let data = Cursor::new(lib_circuits_wrapper);

        // TODO error handling, or at least logging
        let ipfs_result = self.ipfs_client().add(data).await.unwrap();

        let reply = SkcdDisplayReply {
            skcd_cid: ipfs_result.hash,
        };

        Ok(Response::new(reply))
    }

    async fn generate_skcd_generic_from_ipfs(
        &self,
        request: Request<SkcdGenericFromIpfsRequest>,
    ) -> Result<Response<SkcdGenericFromIpfsReply>, Status> {
        log::info!(
            "generate_skcd_generic_from_ipfs request from {:?}",
            request.remote_addr()
        );

        let verilog_cid = &request.get_ref().verilog_cid;

        // get the Verilog (.v) from IPFS
        // DO NOT use dag_get if the file was "add"
        // The returned bytes would be eg
        // {"Data":{"/":{"bytes":"CAISjgQvL....ZfYWRkGI4E"}},"Links":[]}
        // let verilog_buf = self
        //     .ipfs_client()
        //     .dag_get(&verilog_cid)
        //     .map_ok(|chunk| chunk.to_vec())
        //     .try_concat()
        //     .await
        //     .unwrap();
        let verilog_buf = self
            .ipfs_client()
            .cat(verilog_cid)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .unwrap();

        // write the buffer to a file in /tmp
        // yosys/abc REQUIRE file b/c they are basically cli
        // so either write it on Rust side, or send as std::string to C++ and write it there
        let tmp_dir = Builder::new()
            .prefix("interstellar-circuit_routes-generate_skcd_generic_from_ipfs")
            .tempdir()
            .unwrap();
        let verilog_file_path = tmp_dir.path().join("input.v");
        std::fs::write(&verilog_file_path, verilog_buf).expect("could not write");

        // TODO class member/Trait for "lib_circuits_wrapper::ffi::new_circuit_gen_wrapper()"
        let lib_circuits_wrapper = tokio::task::spawn_blocking(move || {
            let wrapper = lib_circuits_wrapper::ffi::new_circuit_gen_wrapper();

            let skcd_pb_buf =
                wrapper.GenerateGenericSkcd(verilog_file_path.as_os_str().to_str().unwrap());

            skcd_pb_buf
        })
        .await
        .unwrap();

        let data = Cursor::new(lib_circuits_wrapper);

        // TODO error handling, or at least logging
        let ipfs_result = self.ipfs_client().add(data).await.unwrap();

        let reply = SkcdGenericFromIpfsReply {
            skcd_cid: ipfs_result.hash,
        };

        Ok(Response::new(reply))
    }
}
