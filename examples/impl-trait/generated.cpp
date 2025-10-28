#include "generated.h"

extern "C" {


  
    
      void _zngur_crate_Greeter_s12_greet(
        uint8_t* data , uint8_t* o
      ) {
        ::rust::crate::Greeter* data_typed = reinterpret_cast< ::rust::crate::Greeter* >(data);
        ::rust::std::string::String oo = data_typed->greet(
          
        );
        ::rust::__zngur_internal_move_to_rust(o, oo);
      }
    
  






} // extern "C"