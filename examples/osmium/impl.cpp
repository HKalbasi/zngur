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
template <typename T> using Option = rust::std::option::Option<T>;
template <typename T> using Ref = rust::Ref<T>;
using rust::std::string::String;

Result<Reader, String> rust::exported_functions::new_reader(Flags f) {
  try {
    Reader o(rust::ZngurCppOpaqueOwnedObject::build<osmium::io::Reader>(
        "map.osm", static_cast<osmium::osm_entity_bits::type>(f.bits())));
    return Result<Reader, String>::Ok(move(o));
  } catch (const exception &ex) {
    return Result<Reader, String>::Err(
        rust::Str::from_char_star(ex.what()).to_string());
  }
}

class RustHandler : public osmium::handler::Handler {
  BendHandler inner;

public:
  void way(osmium::Way &way) { inner.way(way); }

  RustHandler(BendHandler &&inner) : inner(std::move(inner)) {}
};

rust::Tuple<> rust::Impl<Reader>::apply(Ref<Reader> reader,
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

Ref<WayNodeList> rust::Impl<Way>::nodes(Ref<Way> self) {
  return self.cpp().nodes();
}

Ref<TagList> rust::Impl<Way>::tags(Ref<Way> self) { return self.cpp().tags(); }

Option<Ref<rust::Str>>
rust::Impl<TagList>::get_value_by_key(Ref<TagList> self, Ref<rust::Str> key) {
  string cpp_key{(const char *)key.as_ptr(), key.len()};
  auto value = self.cpp().get_value_by_key(cpp_key.c_str());
  if (value == nullptr) {
    return Option<Ref<rust::Str>>::None();
  }
  return Option<Ref<rust::Str>>::Some(rust::Str::from_char_star(value));
}

size_t rust::Impl<WayNodeList>::len(Ref<WayNodeList> self) {
  return self.cpp().size();
}

Ref<Node> rust::Impl<WayNodeList, rust::std::ops::Index<size_t, Node>>::index(
    Ref<WayNodeList> self, size_t i) {
  return self.cpp()[i];
}

double rust::Impl<Node>::distance(Ref<Node> self, Ref<Node> other) {
  return osmium::geom::haversine::distance(self.cpp().location(),
                                           other.cpp().location());
}

String rust::Impl<Node>::href(Ref<Node> self) {
  auto s = String::new_();
  s.push_str(rust::Str::from_char_star("https://www.openstreetmap.org/node/"));
  s.push_str(rust::Str::from_char_star(to_string(self.cpp().ref()).c_str()));
  return s;
}
