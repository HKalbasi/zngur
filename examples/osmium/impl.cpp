#include "./generated.h"

#include <cstddef>
#include <osmium/geom/haversine.hpp>
#include <osmium/handler/node_locations_for_ways.hpp>
#include <osmium/index/map/sparse_mem_array.hpp>
#include <osmium/io/gzip_compression.hpp>
#include <osmium/io/xml_input.hpp>
#include <osmium/osm/entity_bits.hpp>
#include <osmium/osm/way.hpp>
#include <osmium/visitor.hpp>

using namespace rust::crate;
using namespace std;

Reader rust::exported_functions::new_blob_store_client(Flags f) {
  Reader o(rust::ZngurCppOpaqueOwnedObject::build<osmium::io::Reader>(
      "map.osm", static_cast<osmium::osm_entity_bits::type>(f.bits())));
  return o;
}

class RustHandler : public osmium::handler::Handler {
  ::rust::crate::BendHandler inner;

public:
  void way(osmium::Way &way) {
    auto rusty_way = rust::Ref<rust::crate::Way>::build(way);
    inner.way(rusty_way);
  }

  RustHandler(BendHandler &&inner) : inner(std::move(inner)) {}
};

::rust::Unit
rust::exported_functions::apply(::rust::Ref<::rust::crate::Reader> reader,
                                ::rust::crate::BendHandler handler) {
  using IndexType =
      osmium::index::map::SparseMemArray<osmium::unsigned_object_id_type,
                                         osmium::Location>;
  IndexType index;
  auto location_handler =
      osmium::handler::NodeLocationsForWays<IndexType>{index};

  osmium::apply(reader.cpp(), location_handler, RustHandler(move(handler)));
  return {};
}

rust::Ref<::rust::crate::WayNodeList>
rust::Impl<rust::crate::Way>::nodes(rust::Ref<::rust::crate::Way> self) {
  return rust::Ref<::rust::crate::WayNodeList>::build(self.cpp().nodes());
}

rust::Ref<::rust::crate::TagList>
rust::Impl<rust::crate::Way>::tags(rust::Ref<::rust::crate::Way> self) {
  return rust::Ref<::rust::crate::TagList>::build(self.cpp().tags());
}

rust::std::option::Option<rust::Ref<rust::Str>>
rust::Impl<rust::crate::TagList>::get_value_by_key(
    ::rust::Ref<::rust::crate::TagList> self, ::rust::Ref<::rust::Str> key) {
  string cpp_key{(const char *)key.as_ptr(), key.len()};
  auto value = self.cpp().get_value_by_key(cpp_key.c_str());
  if (value == nullptr) {
    return rust::std::option::Option<rust::Ref<rust::Str>>::None();
  }
  return rust::std::option::Option<rust::Ref<rust::Str>>::Some(
      rust::Str::from_char_star(value));
}

size_t rust::Impl<rust::crate::WayNodeList>::len(
    rust::Ref<::rust::crate::WayNodeList> self) {
  return self.cpp().size();
}

rust::Ref<::rust::crate::Node> rust::Impl<rust::crate::WayNodeList>::get(
    rust::Ref<::rust::crate::WayNodeList> self, size_t i) {
  return rust::Ref<::rust::crate::Node>::build(self.cpp()[i]);
}

double
rust::Impl<rust::crate::Node>::distance(rust::Ref<::rust::crate::Node> self,
                                        rust::Ref<::rust::crate::Node> other) {
  return osmium::geom::haversine::distance(self.cpp().location(),
                                           other.cpp().location());
}

rust::std::string::String
rust::Impl<rust::crate::Node>::href(rust::Ref<::rust::crate::Node> self) {
  auto s = rust::std::string::String::new_();
  s.push_str(rust::Str::from_char_star("https://www.openstreetmap.org/node/"));
  s.push_str(rust::Str::from_char_star(to_string(self.cpp().ref()).c_str()));
  return s;
}
