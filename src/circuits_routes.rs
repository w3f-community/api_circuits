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
use interstellarpbapicircuits::skcd_api_server::SkcdApi;
pub use interstellarpbapicircuits::skcd_api_server::SkcdApiServer;
use interstellarpbapicircuits::{
    SkcdDisplayReply, SkcdDisplayRequest, SkcdGenericFromIpfsReply, SkcdGenericFromIpfsRequest,
    SkcdServerMetadata,
};
use ipfs_api_backend_hyper::{
    BackendWithGlobalOptions, GlobalOptions, IpfsApi, IpfsClient, TryFromUri,
};
use std::io::Cursor;
use std::io::Write;
use std::time::Duration;
use tempfile::Builder;
use tonic::{Request, Response, Status};

// https://github.com/neoeinstein/protoc-gen-prost/issues/26
#[allow(clippy::derive_partial_eq_without_eq)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::wildcard_imports)]
#[allow(clippy::doc_markdown)]
#[allow(clippy::similar_names)]
#[allow(clippy::default_trait_access)]
pub mod interstellarpbapicircuits {
    tonic::include_proto!("interstellarpbapicircuits");
}

// #[derive(Default)]
pub struct SkcdApiServerImpl {
    pub ipfs_server_multiaddr: String,
}

trait HasIpfsClient {
    fn ipfs_client(&self) -> Result<BackendWithGlobalOptions<IpfsClient>, Status>;
}

impl HasIpfsClient for SkcdApiServerImpl {
    fn ipfs_client(&self) -> Result<BackendWithGlobalOptions<IpfsClient>, Status> {
        log::info!(
            "ipfs_client: starting with: {}",
            &self.ipfs_server_multiaddr
        );
        Ok(BackendWithGlobalOptions::new(
            ipfs_api_backend_hyper::IpfsClient::from_multiaddr_str(&self.ipfs_server_multiaddr)
                .map_err(|err| Status::invalid_argument(err.to_string()))?,
            GlobalOptions::builder()
                .timeout(Duration::from_millis(5000))
                .build(),
        ))
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
        .map_err(|err| Status::internal(err.to_string()))?;

        let data = Cursor::new(lib_circuits_wrapper.skcd_buffer);

        // TODO error handling, or at least logging
        let ipfs_result = self
            .ipfs_client()?
            .add(data)
            .await
            .map_err(|err| Status::unavailable(err.to_string()))?;

        let reply = SkcdDisplayReply {
            skcd_cid: ipfs_result.hash,
            server_metadata: Some(SkcdServerMetadata {
                // TODO remove this field; the old value was computed from "digits_bboxes"(ie digits_bboxes / 4)
                // so there was no point in passing it from Request to Response
                nb_digits: 0,
            }),
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
            .ipfs_client()?
            .cat(verilog_cid)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .map_err(|err| Status::unavailable(err.to_string()))?;

        // write the buffer to a file in /tmp
        // yosys/abc REQUIRE file b/c they are basically cli
        // so either write it on Rust side, or send as std::string to C++ and write it there
        let tmp_dir = Builder::new()
            .prefix("interstellar-circuit_routes-generate_skcd_generic_from_ipfs")
            .tempdir()
            .map_err(|err| Status::internal(err.to_string()))?;
        let verilog_file_path = tmp_dir.path().join("input.v");
        {
            // MUST drop the file else we get sporadic
            // Entered genlib library with 16 gates from file "/home/xxx/Documents/interstellar/api_circuits/lib_circuits_wrapper/deps/lib_circuits/data/verilog/skcd.genlib".
            // E20230117 13:07:41.909034 26231 verilog_compiler.cpp:59] FilterErrorStreamBuf : Error : ERROR: Can't open input file `/tmp/interstellar-circuit_routes-generate_skcd_generic_from_ipfsQtXDxw/input.v' for reading: No such file or directory
            let mut input_v_file = std::fs::File::create(&verilog_file_path)?;
            input_v_file
                .write_all(&verilog_buf)
                .map_err(|err| Status::unavailable(err.to_string()))?;
        }

        // TODO class member/Trait for "lib_circuits_wrapper::ffi::new_circuit_gen_wrapper()"
        let lib_circuits_wrapper = tokio::task::spawn_blocking(move || {
            let wrapper = lib_circuits_wrapper::ffi::new_circuit_gen_wrapper();

            let skcd_pb_buf = wrapper.GenerateGenericSkcd(
                verilog_file_path
                    .as_os_str()
                    .to_str()
                    .ok_or_else(|| Status::unavailable("as_os_str::to_str FAILED"))?,
            );

            Ok(skcd_pb_buf)
        })
        .await
        .map_err(|err| Status::internal(err.to_string()))?
        .map_err(|err: Status| Status::internal(err.to_string()))?;

        let data = Cursor::new(lib_circuits_wrapper);

        // TODO error handling, or at least logging
        let ipfs_result = self
            .ipfs_client()?
            .add(data)
            .await
            .map_err(|err| Status::unavailable(err.to_string()))?;

        let reply = SkcdGenericFromIpfsReply {
            skcd_cid: ipfs_result.hash,
        };

        Ok(Response::new(reply))
    }
}
