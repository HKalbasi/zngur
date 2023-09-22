#include "./generated.h"
extern "C" {
void __zngur_crate_PrintOnDropConsumer_s13_consume(uint8_t* data, uint8_t* i0, uint8_t* o) {
   ::rust::crate::PrintOnDropConsumer* data_typed = reinterpret_cast<::rust::crate::PrintOnDropConsumer*>(data);
   ::rust::Unit oo = data_typed->consume(::rust::__zngur_internal_move_from_rust<::rust::crate::PrintOnDrop>(i0));   ::rust::__zngur_internal_move_to_rust(o, oo);
}
}
