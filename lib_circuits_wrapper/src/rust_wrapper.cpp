#include "rust_wrapper.h"

#include <functional>

#include "circuit_lib.h"

GenerateDisplaySkcdWrapper::GenerateDisplaySkcdWrapper() {}

rust::Vec<u_int8_t> GenerateDisplaySkcdWrapper::GenerateDisplaySkcd(uint32_t width, uint32_t height) const
{
  auto buf_str = interstellar::CircuitPipeline::GenerateDisplaySkcd(width, height);
  // std::vector<uint8_t> vec(buf_str.begin(), buf_str.end());
  // return vec;
  // return buf_str;
  rust::Vec<u_int8_t> vec;
  std::copy(buf_str.begin(), buf_str.end(), std::back_inserter(vec));
  return vec;
}

rust::Vec<u_int8_t> GenerateDisplaySkcdWrapper::GenerateGenericSkcd(rust::Str verilog_input_path) const
{
  auto buf_str = interstellar::CircuitPipeline::GenerateSkcd({
      std::string(verilog_input_path),
  });
  // std::vector<uint8_t> vec(buf_str.begin(), buf_str.end());
  // return vec;
  // return buf_str;
  rust::Vec<u_int8_t> vec;
  std::copy(buf_str.begin(), buf_str.end(), std::back_inserter(vec));
  return vec;
}

std::unique_ptr<GenerateDisplaySkcdWrapper> new_circuit_gen_wrapper()
{
  return std::make_unique<GenerateDisplaySkcdWrapper>();
}

// #include "cxx-demo/include/blobstore.h"
// #include "cxx-demo/src/main.rs.h"
// #include <functional>

// BlobstoreClient::BlobstoreClient() {}

// // Upload a new blob and return a blobid that serves as a handle to the blob.
// uint64_t BlobstoreClient::put(MultiBuf &buf) const
// {
//   // Traverse the caller's chunk iterator.
//   std::string contents;
//   while (true)
//   {
//     auto chunk = next_chunk(buf);
//     if (chunk.size() == 0)
//     {
//       break;
//     }
//     contents.append(reinterpret_cast<const char *>(chunk.data()),
//     chunk.size());
//   }

//   // Pretend we did something useful to persist the data.
//   auto blobid = std::hash<std::string>{}(contents);
//   return blobid;
// }

// std::unique_ptr<BlobstoreClient> new_blobstore_client()
// {
//   return std::unique_ptr<BlobstoreClient>(new BlobstoreClient());
// }
