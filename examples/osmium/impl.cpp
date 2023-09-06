#include "./generated.h"

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
  Reader o(rust::ZngurCppOpaqueObject::build<osmium::io::Reader>(
      "map.osm", static_cast<osmium::osm_entity_bits::type>(f.bits())));
  return o;
}

class RustHandler : public osmium::handler::Handler {
  ::rust::crate::BendHandler inner;

public:
  void way(const osmium::Way &way) {
    rust::crate::Way rusty_way;
    // rust::ZngurCppOpaqueObject::build<osmium::Way>(way));
    inner.way(rusty_way);
  }

  RustHandler(BendHandler &&inner) : inner(std::move(inner)) {}
};

::rust::Unit
rust::exported_functions::apply(::rust::Ref<::rust::crate::Reader> reader,
                                ::rust::crate::BendHandler handler) {
  std::cout << "hello" << std::endl;
  osmium::apply(reader.cpp(), RustHandler(std::move(handler)));
  return {};
}
