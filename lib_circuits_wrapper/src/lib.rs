// #![no_std]
// https://github.com/substrate-developer-hub/substrate-module-template/blob/master/HOWTO.md#forgetting-cfg_attr-for-no_std
#![cfg_attr(not(feature = "std"), no_std)]

// extern crate std;

// extern crate alloc;
// extern crate sp_std;

// TODO simplify; but something is needed to avoid
// Vec<u8>;
//    |  ^^^ could not find `std` in the list of imported crates
// use sp_std::{
//     marker::PhantomData,
//     ops::{Deref, DerefMut},
//     prelude::*,
// };
// use sp_std::vec::Vec;

pub use cxx;

#[cxx::bridge]
pub mod ffi {
    // extern "Rust" {
    //     type MultiBuf;

    //     fn next_chunk(buf: &mut MultiBuf) -> &[u8];
    // }

    unsafe extern "C++" {
        include!("lib-circuits-wrapper/src/rust_wrapper.h");

        type GenerateDisplaySkcdWrapper;

        fn new_circuit_gen_wrapper() -> UniquePtr<GenerateDisplaySkcdWrapper>;
        fn GenerateDisplaySkcd(&self, output_skcd_path: &str, width: u32, height: u32);
    }
}

// // An iterator over contiguous chunks of a discontiguous file object. Toy
// // implementation uses a Vec<Vec<u8>> but in reality this might be iterating
// // over some more complex Rust data structure like a rope, or maybe loading
// // chunks lazily from somewhere.
// pub struct MultiBuf {
//     chunks: Vec<Vec<u8>>,
//     pos: usize,
// }

// pub fn next_chunk(buf: &mut MultiBuf) -> &[u8] {
//     let next = buf.chunks.get(buf.pos);
//     buf.pos += 1;
//     next.map_or(&[], Vec::as_slice)
// }

#[cfg(test)]
mod tests {
    use crate::ffi;
    use std::fs::File;
    use tempfile::Builder;

    // TODO fix undefined reference to `GenerateSegmentedDigitCache()' aaa
    #[test]
    fn generate_display_skcd_basic() {
        let circuit_gen_wrapper = ffi::new_circuit_gen_wrapper();

        let width = 224;
        let height = 96;

        let tmp_dir = Builder::new()
            .prefix("interstellar-circuit_routes")
            .tempdir()
            .unwrap();

        let file_path = tmp_dir.path().join("output.skcd.pb.bin");

        // TODO make the C++ API return a buffer?
        circuit_gen_wrapper.GenerateDisplaySkcd(
            file_path.as_os_str().to_str().unwrap(),
            width,
            height,
        );

        // TODO test file_path size? just exists?
        assert!(file_path.exists());
        assert_eq!(file_path.metadata().unwrap().len(), 4242);
    }
}
