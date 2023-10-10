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
template <typename T, typename E>
using Result = rust::std::result::Result<T, E>;

Result<Reader, rust::std::string::String>
rust::exported_functions::new_reader(Flags f) {
  try {
    Reader o(rust::ZngurCppOpaqueOwnedObject::build<osmium::io::Reader>(
        "map.osm", static_cast<osmium::osm_entity_bits::type>(f.bits())));
    return Result<Reader, rust::std::string::String>::Ok(move(o));
  } catch (const exception &ex) {
    return Result<Reader, rust::std::string::String>::Err(
        rust::Str::from_char_star(ex.what()).to_string());
  }
}

class RustHandler : public osmium::handler::Handler {
  BendHandler inner;

public:
  void way(osmium::Way &way) { inner.way(way); }

  RustHandler(BendHandler &&inner) : inner(std::move(inner)) {}
};

rust::Tuple<> rust::Impl<Reader>::apply(rust::Ref<Reader> reader,
                                        BendHandler handler) {
  using IndexType =
      osmium::index::map::SparseMemArray<osmium::unsigned_object_id_type,
                                         osmium::Location>;
  IndexType index;
  auto location_handler =
      osmium::handler::NodeLocationsForWays<IndexType>{index};

  osmium::apply(reader.cpp(), location_handler, RustHandler(move(handler)));
  return {};
}

rust::Ref<WayNodeList> rust::Impl<Way>::nodes(rust::Ref<Way> self) {
  return self.cpp().nodes();
}

rust::Ref<TagList> rust::Impl<Way>::tags(rust::Ref<Way> self) {
  return self.cpp().tags();
}

rust::std::option::Option<rust::Ref<rust::Str>>
rust::Impl<TagList>::get_value_by_key(rust::Ref<TagList> self,
                                      rust::Ref<rust::Str> key) {
  string cpp_key{(const char *)key.as_ptr(), key.len()};
  auto value = self.cpp().get_value_by_key(cpp_key.c_str());
  if (value == nullptr) {
    return rust::std::option::Option<rust::Ref<rust::Str>>::None();
  }
  return rust::std::option::Option<rust::Ref<rust::Str>>::Some(
      rust::Str::from_char_star(value));
}

size_t rust::Impl<WayNodeList>::len(rust::Ref<WayNodeList> self) {
  return self.cpp().size();
}

rust::Ref<Node>
rust::Impl<WayNodeList, rust::std::ops::Index<size_t, Node>>::index(
    rust::Ref<WayNodeList> self, size_t i) {
  return self.cpp()[i];
}

double rust::Impl<Node>::distance(rust::Ref<Node> self, rust::Ref<Node> other) {
  return osmium::geom::haversine::distance(self.cpp().location(),
                                           other.cpp().location());
}

rust::std::string::String rust::Impl<Node>::href(rust::Ref<Node> self) {
  auto s = rust::std::string::String::new_();
  s.push_str(rust::Str::from_char_star("https://www.openstreetmap.org/node/"));
  s.push_str(rust::Str::from_char_star(to_string(self.cpp().ref()).c_str()));
  return s;
}
