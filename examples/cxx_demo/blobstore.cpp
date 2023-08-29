#include <algorithm>
#include <set>
#include <unordered_map>

#include "./generated.h"

class BlobStore : ::rust::Impl<::rust::crate::BlobStoreTrait> {
  class Impl {
    friend BlobStore;

    using Blob = struct {
      std::string data;
      std::set<std::string> tags;
    };
    std::unordered_map<uint64_t, Blob> blobs;
  };

  uint64_t put(::rust::Ref<::rust::crate::MultiBuf> i0) override { return 5; }

  ::rust::Unit tag(::uint64_t blob_id,
                   ::rust::Ref<::rust::core::primitive::str> tag) override {
    impl.blobs[blob_id].tags.emplace(tag);
    return ::rust::Unit{};
  }

  ::rust::crate::BlobMetadata metadata(::uint64_t i0) override {
    ::rust::crate::BlobMetadata r = ::rust::crate::BlobMetadata::default_();
    return r;
  }

private:
  Impl impl;
};
