#include "./generated.h"

#include <osmium/handler/node_locations_for_ways.hpp>
#include <osmium/index/map/sparse_mem_array.hpp>
#include <osmium/io/gzip_compression.hpp>
#include <osmium/io/xml_input.hpp>
#include <osmium/osm/entity_bits.hpp>
#include <osmium/visitor.hpp>

using namespace rust::crate;
using namespace std;

Reader rust::exported_functions::new_blob_store_client(Flags f) {
  osmium::io::Reader reader{
      "map.osm", static_cast<osmium::osm_entity_bits::type>(f.bits())};
  cout << "hello world " << reader.file_size() << endl;
  Reader o(6);
  return o;
}
