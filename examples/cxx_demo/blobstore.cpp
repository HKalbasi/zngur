#include <algorithm>
#include <cstdint>
#include <set>
#include <unordered_map>

#include "./generated.h"

class BlobStore : public ::rust::crate::BlobStoreTrait {
  class Impl {
    friend BlobStore;

    using Blob = struct {
      std::string data;
      std::set<std::string> tags;
    };
    std::unordered_map<uint64_t, Blob> blobs;
  };

public:
  uint64_t put(::rust::Ref<::rust::crate::MultiBuf> buf) override {
    std::string contents;

    // Traverse the caller's chunk iterator.
    //
    // In reality there might be sophisticated batching of chunks and/or
    // parallel upload implemented by the blob_store's C++ client.
    while (true) {
      auto chunk = buf.next_chunk();
      if (chunk.len() == 0) {
        break;
      }
      contents.append(reinterpret_cast<const char *>(chunk.as_ptr()),
                      chunk.len());
    }

    // Insert into map and provide caller the handle.
    auto blob_id = std::hash<std::string>{}(contents);
    impl.blobs[blob_id] = {std::move(contents), {}};
    return blob_id;
  }

  ::rust::Unit tag(::uint64_t blob_id,
                   ::rust::Ref<::rust::core::primitive::str> tag) override {
    impl.blobs[blob_id].tags.emplace((char *)tag.as_ptr(), tag.len());
    return ::rust::Unit{};
  }

  ::rust::crate::BlobMetadata metadata(::uint64_t blob_id) override {
    ::rust::crate::BlobMetadata r = ::rust::crate::BlobMetadata::default_();
    auto blob = impl.blobs.find(blob_id);
    if (blob != impl.blobs.end()) {
      r.set_size(blob->second.data.size());
      std::for_each(blob->second.tags.cbegin(), blob->second.tags.cend(),
                    [&](auto &t) { r.push_tag((int8_t *)t.c_str()); });
    }
    return r;
  }

private:
  Impl impl;
};

::rust::Box<::rust::Dyn<::rust::crate::BlobStoreTrait>>
rust::exported_functions::new_blob_store_client() {
  return ::rust::Box<::rust::Dyn<::rust::crate::BlobStoreTrait>>::make_box<
      BlobStore>();
}
