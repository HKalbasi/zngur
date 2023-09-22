#include "./generated.h"
extern "C" {
void __zngur__std_iter_Iterator_Item_i32__s8s12s17m26e31y35_next(uint8_t* data, uint8_t* o) {
   ::rust::std::iter::Iterator<::int32_t>* data_typed = reinterpret_cast<::rust::std::iter::Iterator<::int32_t>*>(data);
   ::rust::std::option::Option<::int32_t> oo = data_typed->next();   ::rust::__zngur_internal_move_to_rust(o, oo);
}
}
