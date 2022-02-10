// wrapper for our "lib_server" circuit generator

#pragma once

#include <memory>

#include "rust/cxx.h"

/**
 * Wrapper around interstellar::CircuitPipeline::GenerateDisplaySkcd
 */
class GenerateDisplaySkcdWrapper
{
public:
  GenerateDisplaySkcdWrapper();

  // TODO not const, but will certainly break Rust's no_std CXX side():
  // self: Pin<&mut GenerateDisplaySkcdWrapper>,
  //  ^^^ could not find `std` in the list of imported crates
  void GenerateDisplaySkcd(rust::Str output_skcd_path, uint32_t width, uint32_t height) const;

private:
  // TODO dynamic
  bool allow_cache_ = false;
};

std::unique_ptr<GenerateDisplaySkcdWrapper> new_circuit_gen_wrapper();