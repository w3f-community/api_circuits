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

#include "rust_wrapper.h"

#include <functional>

#include "circuit_lib.h"

// generated
// needed only if shared structs
#include "lib-circuits-wrapper/src/lib.rs.h"

GenerateDisplaySkcdWrapper::GenerateDisplaySkcdWrapper() {}

SkcdAndMetadata GenerateDisplaySkcdWrapper::GenerateDisplaySkcd(uint32_t width, uint32_t height,
                                                                // DisplayDigitType digit_type,
                                                                // const rust::Vec<BBox> &digits_bboxes
                                                                const rust::Vec<float> &digits_bboxes) const
{
  // CHECK: digits_bboxes SHOULD be a list ob bboxes, passed as (x1,y1,x2,y2)
  size_t digits_bboxes_size = digits_bboxes.size();
  if (!digits_bboxes_size % 4)
  {
    throw std::invalid_argument("GenerateDisplaySkcd: digits_bboxes must be a list of bboxes(ie size == mod 4)");
  }
  std::vector<std::tuple<float, float, float, float>> digits_bboxes_copy;
  digits_bboxes_copy.reserve(digits_bboxes_size / 4);
  for (uint32_t i = 0; i < digits_bboxes_size; i += 4)
  {
    digits_bboxes_copy.emplace_back(digits_bboxes[i], digits_bboxes[i + 1],
                                    digits_bboxes[i + 2], digits_bboxes[i + 3]);
  }

  auto buf_str = interstellar::circuits::GenerateDisplaySkcd(width, height,
                                                             interstellar::circuits::DisplayDigitType::seven_segments_png,
                                                             std::move(digits_bboxes_copy));

  rust::Vec<u_int8_t> vec;
  std::copy(buf_str.begin(), buf_str.end(), std::back_inserter(vec));

  SkcdAndMetadata skcd_and_metadata;
  skcd_and_metadata.skcd_buffer = vec;
  return skcd_and_metadata;
}

rust::Vec<u_int8_t> GenerateDisplaySkcdWrapper::GenerateGenericSkcd(rust::Str verilog_input_path) const
{
  auto buf_str = interstellar::circuits::GenerateSkcd({
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
