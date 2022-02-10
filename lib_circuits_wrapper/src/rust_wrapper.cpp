#include "rust_wrapper.h"

#include <functional>

#include "circuit_lib.h"

GenerateDisplaySkcdWrapper::GenerateDisplaySkcdWrapper() {}

void GenerateDisplaySkcdWrapper::GenerateDisplaySkcd(rust::Str output_skcd_path,
                                                     uint32_t width, uint32_t height) const
{

  interstellar::CircuitPipeline::GenerateDisplaySkcd(boost::filesystem::path(std::string(output_skcd_path)),
                                                     width,
                                                     height);
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
