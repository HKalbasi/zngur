#include "./generated.h"
extern "C" {
void __zngur_new_blob_store_client_(uint8_t* i0,uint8_t* o){
   ::rust::crate::Reader oo = ::rust::exported_functions::new_blob_store_client(::rust::__zngur_internal_move_from_rust<::rust::crate::Flags>(i0));
   ::rust::__zngur_internal_move_to_rust(o, oo);
}
}
