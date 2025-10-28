#pragma once

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <csignal>
#include <array>
#include <iostream>
#include <functional>
#include <math.h>




  namespace rust {
      class Panic {};
  }
  extern "C" {
      uint8_t _zngur__detect_panic_z7();
      void _zngur__take_panic_z7();
  }


#define zngur_dbg(x) (::rust::zngur_dbg_impl(__FILE__, __LINE__, #x, x))

namespace rust {
  template<typename T>
  uint8_t* __zngur_internal_data_ptr(const T& t) ;

  template<typename T>
  void __zngur_internal_assume_init(T& t) ;

  template<typename T>
  void __zngur_internal_assume_deinit(T& t) ;

  template<typename T>
  inline size_t __zngur_internal_size_of() ;

  template<typename T>
  inline void __zngur_internal_move_to_rust(uint8_t* dst, T& t) {
    memcpy(dst, ::rust::__zngur_internal_data_ptr(t), ::rust::__zngur_internal_size_of<T>());
    ::rust::__zngur_internal_assume_deinit(t);
  }

  template<typename T>
  inline T __zngur_internal_move_from_rust(uint8_t* src) {
    T t;
    ::rust::__zngur_internal_assume_init(t);
    memcpy(::rust::__zngur_internal_data_ptr(t), src, ::rust::__zngur_internal_size_of<T>());
    return t;
  }

  template<typename T>
  inline void __zngur_internal_check_init(const T&) {}

  class ZngurCppOpaqueOwnedObject {
    uint8_t* data;
    void (*destructor)(uint8_t*);

  public:
    template<typename T, typename... Args>
    inline static ZngurCppOpaqueOwnedObject build(Args&&... args) {
        ZngurCppOpaqueOwnedObject o;
        o.data = reinterpret_cast<uint8_t*>(new T(::std::forward<Args>(args)...));
        o.destructor = [](uint8_t* d) {
            delete reinterpret_cast<T*>(d);
        };
        return o;
    }

    template<typename T>
    inline T& as_cpp() { return *reinterpret_cast<T *>(data); }
  };

  template<typename T>
  struct Ref;

  template<typename T>
  struct RefMut;

  template<typename T, size_t OFFSET>
  struct FieldOwned {
    inline operator T() const { return *::rust::Ref<T>(*this); }
  };

  template<typename T, size_t OFFSET>
  struct FieldRef {
    inline operator T() const { return *::rust::Ref<T>(*this); }
  };

  template<typename T, size_t OFFSET>
  struct FieldRefMut {
    inline operator T() const { return *::rust::Ref<T>(*this); }
  };

  template<typename T>
  struct zngur_is_unsized : std::false_type {};
  struct zngur_fat_pointer {
    uint8_t* data;
    size_t metadata;
  };
  template<typename T>
  struct Raw {
      using DataType = typename std::conditional<
          zngur_is_unsized<T>::value,
          zngur_fat_pointer,
          uint8_t*
      >::type;
      DataType data;
      Raw() {}
      Raw(Ref<T> value) {
          memcpy(&data, __zngur_internal_data_ptr<Ref<T>>(value), __zngur_internal_size_of<Ref<T>>());
      }
      Raw(RefMut<T> value) {
          memcpy(&data, __zngur_internal_data_ptr<RefMut<T>>(value), __zngur_internal_size_of<RefMut<T>>());
      }
      Raw(DataType data) : data(data) {
      }
      Raw<T> offset(ssize_t n) {
          return Raw(data + n * __zngur_internal_size_of<T>());
      }
      Ref<T> read_ref() {
          Ref<T> value;
          memcpy(__zngur_internal_data_ptr<Ref<T>>(value), &data, __zngur_internal_size_of<Ref<T>>());
          __zngur_internal_assume_init<Ref<T>>(value);
          return value;
      }
  };
  template<typename T>
  struct RawMut {
      using DataType = typename std::conditional<
          zngur_is_unsized<T>::value,
          zngur_fat_pointer,
          uint8_t*
      >::type;
      DataType data;
      RawMut() {}
      RawMut(RefMut<T> value) {
          memcpy(&data, __zngur_internal_data_ptr<RefMut<T>>(value), __zngur_internal_size_of<RefMut<T>>());
      }
      RawMut(DataType data) : data(data) {
      }
      RawMut<T> offset(ssize_t n) {
          return RawMut(data + n * __zngur_internal_size_of<T>());
      }
      T read() {
          T value;
          memcpy(__zngur_internal_data_ptr<T>(value), data, __zngur_internal_size_of<T>());
          __zngur_internal_assume_init<T>(value);
          return value;
      }
      Ref<T> read_ref() {
          Ref<T> value;
          memcpy(__zngur_internal_data_ptr<Ref<T>>(value), &data, __zngur_internal_size_of<Ref<T>>());
          __zngur_internal_assume_init<Ref<T>>(value);
          return value;
      }
      RefMut<T> read_mut() {
          RefMut<T> value;
          memcpy(__zngur_internal_data_ptr<RefMut<T>>(value), &data, __zngur_internal_size_of<RefMut<T>>());
          __zngur_internal_assume_init<RefMut<T>>(value);
          return value;
      }
      void write(T value) {
          memcpy(data, __zngur_internal_data_ptr<T>(value), __zngur_internal_size_of<T>());
          __zngur_internal_assume_deinit<T>(value);
      }
  };
  template<typename... T>
  struct Tuple;

  using Unit = Tuple<>;

  template<typename T>
  struct ZngurPrettyPrinter;

  class Inherent;

  template<typename Type, typename Trait = Inherent>
  class Impl;

  template<typename T>
  T&& zngur_dbg_impl(const char* file_name, int line_number, const char* exp, T&& input) {
    ::std::cerr << "[" << file_name << ":" << line_number << "] " << exp << " = ";
    ZngurPrettyPrinter<typename ::std::remove_reference<T>::type>::print(input);
    return ::std::forward<T>(input);
  }


  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int8_t >(const int8_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int8_t >(int8_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int8_t >(int8_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< int8_t >() {
    return sizeof(int8_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int8_t*>(int8_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int8_t*>(int8_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int8_t*>(int8_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int8_t const*>(int8_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int8_t const*>(int8_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int8_t const*>(int8_t const*&) {}

  template<>
  struct Ref< int8_t > {
    Ref() {
      data = 0;
    }
    Ref(const int8_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< int8_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< int8_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< int8_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    int8_t& operator*() {
      return *reinterpret_cast< int8_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< int8_t > >(const ::rust::Ref< int8_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< int8_t > >;

  };

  template<>
  struct RefMut< int8_t > {
    RefMut() {
      data = 0;
    }

    RefMut(int8_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< int8_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< int8_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    int8_t& operator*() {
        return *reinterpret_cast< int8_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< int8_t > >(const ::rust::RefMut< int8_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< int8_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< int8_t > {
      static inline void print(int8_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint8_t >(const uint8_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint8_t >(uint8_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint8_t >(uint8_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< uint8_t >() {
    return sizeof(uint8_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint8_t*>(uint8_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint8_t*>(uint8_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint8_t*>(uint8_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint8_t const*>(uint8_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint8_t const*>(uint8_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint8_t const*>(uint8_t const*&) {}

  template<>
  struct Ref< uint8_t > {
    Ref() {
      data = 0;
    }
    Ref(const uint8_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< uint8_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< uint8_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< uint8_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    uint8_t& operator*() {
      return *reinterpret_cast< uint8_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< uint8_t > >(const ::rust::Ref< uint8_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< uint8_t > >;

  };

  template<>
  struct RefMut< uint8_t > {
    RefMut() {
      data = 0;
    }

    RefMut(uint8_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< uint8_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< uint8_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    uint8_t& operator*() {
        return *reinterpret_cast< uint8_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< uint8_t > >(const ::rust::RefMut< uint8_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< uint8_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< uint8_t > {
      static inline void print(uint8_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int16_t >(const int16_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int16_t >(int16_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int16_t >(int16_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< int16_t >() {
    return sizeof(int16_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int16_t*>(int16_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int16_t*>(int16_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int16_t*>(int16_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int16_t const*>(int16_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int16_t const*>(int16_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int16_t const*>(int16_t const*&) {}

  template<>
  struct Ref< int16_t > {
    Ref() {
      data = 0;
    }
    Ref(const int16_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< int16_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< int16_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< int16_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    int16_t& operator*() {
      return *reinterpret_cast< int16_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< int16_t > >(const ::rust::Ref< int16_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< int16_t > >;

  };

  template<>
  struct RefMut< int16_t > {
    RefMut() {
      data = 0;
    }

    RefMut(int16_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< int16_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< int16_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    int16_t& operator*() {
        return *reinterpret_cast< int16_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< int16_t > >(const ::rust::RefMut< int16_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< int16_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< int16_t > {
      static inline void print(int16_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint16_t >(const uint16_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint16_t >(uint16_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint16_t >(uint16_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< uint16_t >() {
    return sizeof(uint16_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint16_t*>(uint16_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint16_t*>(uint16_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint16_t*>(uint16_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint16_t const*>(uint16_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint16_t const*>(uint16_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint16_t const*>(uint16_t const*&) {}

  template<>
  struct Ref< uint16_t > {
    Ref() {
      data = 0;
    }
    Ref(const uint16_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< uint16_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< uint16_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< uint16_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    uint16_t& operator*() {
      return *reinterpret_cast< uint16_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< uint16_t > >(const ::rust::Ref< uint16_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< uint16_t > >;

  };

  template<>
  struct RefMut< uint16_t > {
    RefMut() {
      data = 0;
    }

    RefMut(uint16_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< uint16_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< uint16_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    uint16_t& operator*() {
        return *reinterpret_cast< uint16_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< uint16_t > >(const ::rust::RefMut< uint16_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< uint16_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< uint16_t > {
      static inline void print(uint16_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int32_t >(const int32_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int32_t >(int32_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int32_t >(int32_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< int32_t >() {
    return sizeof(int32_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int32_t*>(int32_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int32_t*>(int32_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int32_t*>(int32_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int32_t const*>(int32_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int32_t const*>(int32_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int32_t const*>(int32_t const*&) {}

  template<>
  struct Ref< int32_t > {
    Ref() {
      data = 0;
    }
    Ref(const int32_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< int32_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< int32_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< int32_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    int32_t& operator*() {
      return *reinterpret_cast< int32_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< int32_t > >(const ::rust::Ref< int32_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< int32_t > >;

  };

  template<>
  struct RefMut< int32_t > {
    RefMut() {
      data = 0;
    }

    RefMut(int32_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< int32_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< int32_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    int32_t& operator*() {
        return *reinterpret_cast< int32_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< int32_t > >(const ::rust::RefMut< int32_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< int32_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< int32_t > {
      static inline void print(int32_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint32_t >(const uint32_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint32_t >(uint32_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint32_t >(uint32_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< uint32_t >() {
    return sizeof(uint32_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint32_t*>(uint32_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint32_t*>(uint32_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint32_t*>(uint32_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint32_t const*>(uint32_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint32_t const*>(uint32_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint32_t const*>(uint32_t const*&) {}

  template<>
  struct Ref< uint32_t > {
    Ref() {
      data = 0;
    }
    Ref(const uint32_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< uint32_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< uint32_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< uint32_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    uint32_t& operator*() {
      return *reinterpret_cast< uint32_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< uint32_t > >(const ::rust::Ref< uint32_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< uint32_t > >;

  };

  template<>
  struct RefMut< uint32_t > {
    RefMut() {
      data = 0;
    }

    RefMut(uint32_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< uint32_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< uint32_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    uint32_t& operator*() {
        return *reinterpret_cast< uint32_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< uint32_t > >(const ::rust::RefMut< uint32_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< uint32_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< uint32_t > {
      static inline void print(uint32_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int64_t >(const int64_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int64_t >(int64_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int64_t >(int64_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< int64_t >() {
    return sizeof(int64_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int64_t*>(int64_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int64_t*>(int64_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int64_t*>(int64_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< int64_t const*>(int64_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< int64_t const*>(int64_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< int64_t const*>(int64_t const*&) {}

  template<>
  struct Ref< int64_t > {
    Ref() {
      data = 0;
    }
    Ref(const int64_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< int64_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< int64_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< int64_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    int64_t& operator*() {
      return *reinterpret_cast< int64_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< int64_t > >(const ::rust::Ref< int64_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< int64_t > >;

  };

  template<>
  struct RefMut< int64_t > {
    RefMut() {
      data = 0;
    }

    RefMut(int64_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< int64_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< int64_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    int64_t& operator*() {
        return *reinterpret_cast< int64_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< int64_t > >(const ::rust::RefMut< int64_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< int64_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< int64_t > {
      static inline void print(int64_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint64_t >(const uint64_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint64_t >(uint64_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint64_t >(uint64_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< uint64_t >() {
    return sizeof(uint64_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint64_t*>(uint64_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint64_t*>(uint64_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint64_t*>(uint64_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< uint64_t const*>(uint64_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< uint64_t const*>(uint64_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< uint64_t const*>(uint64_t const*&) {}

  template<>
  struct Ref< uint64_t > {
    Ref() {
      data = 0;
    }
    Ref(const uint64_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< uint64_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< uint64_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< uint64_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    uint64_t& operator*() {
      return *reinterpret_cast< uint64_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< uint64_t > >(const ::rust::Ref< uint64_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< uint64_t > >;

  };

  template<>
  struct RefMut< uint64_t > {
    RefMut() {
      data = 0;
    }

    RefMut(uint64_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< uint64_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< uint64_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    uint64_t& operator*() {
        return *reinterpret_cast< uint64_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< uint64_t > >(const ::rust::RefMut< uint64_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< uint64_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< uint64_t > {
      static inline void print(uint64_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int8_t> >(const ::rust::Ref<int8_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int8_t> >(::rust::Ref<int8_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int8_t> >(::rust::Ref<int8_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::Ref<int8_t> >() {
    return sizeof(::rust::Ref<int8_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int8_t>*>(::rust::Ref<int8_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int8_t>*>(::rust::Ref<int8_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int8_t>*>(::rust::Ref<int8_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int8_t> const*>(::rust::Ref<int8_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int8_t> const*>(::rust::Ref<int8_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int8_t> const*>(::rust::Ref<int8_t> const*&) {}

  template<>
  struct Ref< ::rust::Ref<int8_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<int8_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<int8_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<int8_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<int8_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<int8_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<int8_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::Ref<int8_t> > >(const ::rust::Ref< ::rust::Ref<int8_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int8_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<int8_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<int8_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<int8_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<int8_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<int8_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<int8_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::Ref<int8_t> > >(const ::rust::RefMut< ::rust::Ref<int8_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int8_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint8_t> >(const ::rust::Ref<uint8_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint8_t> >(::rust::Ref<uint8_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint8_t> >(::rust::Ref<uint8_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::Ref<uint8_t> >() {
    return sizeof(::rust::Ref<uint8_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint8_t>*>(::rust::Ref<uint8_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint8_t>*>(::rust::Ref<uint8_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint8_t>*>(::rust::Ref<uint8_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint8_t> const*>(::rust::Ref<uint8_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint8_t> const*>(::rust::Ref<uint8_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint8_t> const*>(::rust::Ref<uint8_t> const*&) {}

  template<>
  struct Ref< ::rust::Ref<uint8_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<uint8_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<uint8_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<uint8_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<uint8_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<uint8_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<uint8_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::Ref<uint8_t> > >(const ::rust::Ref< ::rust::Ref<uint8_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint8_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<uint8_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<uint8_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<uint8_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<uint8_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<uint8_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<uint8_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::Ref<uint8_t> > >(const ::rust::RefMut< ::rust::Ref<uint8_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint8_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int8_t> >(const ::rust::RefMut<int8_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int8_t> >(::rust::RefMut<int8_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int8_t> >(::rust::RefMut<int8_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::RefMut<int8_t> >() {
    return sizeof(::rust::RefMut<int8_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int8_t>*>(::rust::RefMut<int8_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int8_t>*>(::rust::RefMut<int8_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int8_t>*>(::rust::RefMut<int8_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int8_t> const*>(::rust::RefMut<int8_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int8_t> const*>(::rust::RefMut<int8_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int8_t> const*>(::rust::RefMut<int8_t> const*&) {}

  template<>
  struct Ref< ::rust::RefMut<int8_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<int8_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<int8_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<int8_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<int8_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<int8_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<int8_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::RefMut<int8_t> > >(const ::rust::Ref< ::rust::RefMut<int8_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int8_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<int8_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<int8_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<int8_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<int8_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<int8_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<int8_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::RefMut<int8_t> > >(const ::rust::RefMut< ::rust::RefMut<int8_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int8_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint8_t> >(const ::rust::RefMut<uint8_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint8_t> >(::rust::RefMut<uint8_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint8_t> >(::rust::RefMut<uint8_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::RefMut<uint8_t> >() {
    return sizeof(::rust::RefMut<uint8_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint8_t>*>(::rust::RefMut<uint8_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint8_t>*>(::rust::RefMut<uint8_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint8_t>*>(::rust::RefMut<uint8_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint8_t> const*>(::rust::RefMut<uint8_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint8_t> const*>(::rust::RefMut<uint8_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint8_t> const*>(::rust::RefMut<uint8_t> const*&) {}

  template<>
  struct Ref< ::rust::RefMut<uint8_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<uint8_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<uint8_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<uint8_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<uint8_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<uint8_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<uint8_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::RefMut<uint8_t> > >(const ::rust::Ref< ::rust::RefMut<uint8_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint8_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<uint8_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<uint8_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<uint8_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<uint8_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<uint8_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<uint8_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::RefMut<uint8_t> > >(const ::rust::RefMut< ::rust::RefMut<uint8_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint8_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int16_t> >(const ::rust::Ref<int16_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int16_t> >(::rust::Ref<int16_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int16_t> >(::rust::Ref<int16_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::Ref<int16_t> >() {
    return sizeof(::rust::Ref<int16_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int16_t>*>(::rust::Ref<int16_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int16_t>*>(::rust::Ref<int16_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int16_t>*>(::rust::Ref<int16_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int16_t> const*>(::rust::Ref<int16_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int16_t> const*>(::rust::Ref<int16_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int16_t> const*>(::rust::Ref<int16_t> const*&) {}

  template<>
  struct Ref< ::rust::Ref<int16_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<int16_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<int16_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<int16_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<int16_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<int16_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<int16_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::Ref<int16_t> > >(const ::rust::Ref< ::rust::Ref<int16_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int16_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<int16_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<int16_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<int16_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<int16_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<int16_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<int16_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::Ref<int16_t> > >(const ::rust::RefMut< ::rust::Ref<int16_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int16_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint16_t> >(const ::rust::Ref<uint16_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint16_t> >(::rust::Ref<uint16_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint16_t> >(::rust::Ref<uint16_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::Ref<uint16_t> >() {
    return sizeof(::rust::Ref<uint16_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint16_t>*>(::rust::Ref<uint16_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint16_t>*>(::rust::Ref<uint16_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint16_t>*>(::rust::Ref<uint16_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint16_t> const*>(::rust::Ref<uint16_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint16_t> const*>(::rust::Ref<uint16_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint16_t> const*>(::rust::Ref<uint16_t> const*&) {}

  template<>
  struct Ref< ::rust::Ref<uint16_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<uint16_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<uint16_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<uint16_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<uint16_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<uint16_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<uint16_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::Ref<uint16_t> > >(const ::rust::Ref< ::rust::Ref<uint16_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint16_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<uint16_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<uint16_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<uint16_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<uint16_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<uint16_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<uint16_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::Ref<uint16_t> > >(const ::rust::RefMut< ::rust::Ref<uint16_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint16_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int16_t> >(const ::rust::RefMut<int16_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int16_t> >(::rust::RefMut<int16_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int16_t> >(::rust::RefMut<int16_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::RefMut<int16_t> >() {
    return sizeof(::rust::RefMut<int16_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int16_t>*>(::rust::RefMut<int16_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int16_t>*>(::rust::RefMut<int16_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int16_t>*>(::rust::RefMut<int16_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int16_t> const*>(::rust::RefMut<int16_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int16_t> const*>(::rust::RefMut<int16_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int16_t> const*>(::rust::RefMut<int16_t> const*&) {}

  template<>
  struct Ref< ::rust::RefMut<int16_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<int16_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<int16_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<int16_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<int16_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<int16_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<int16_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::RefMut<int16_t> > >(const ::rust::Ref< ::rust::RefMut<int16_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int16_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<int16_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<int16_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<int16_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<int16_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<int16_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<int16_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::RefMut<int16_t> > >(const ::rust::RefMut< ::rust::RefMut<int16_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int16_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint16_t> >(const ::rust::RefMut<uint16_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint16_t> >(::rust::RefMut<uint16_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint16_t> >(::rust::RefMut<uint16_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::RefMut<uint16_t> >() {
    return sizeof(::rust::RefMut<uint16_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint16_t>*>(::rust::RefMut<uint16_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint16_t>*>(::rust::RefMut<uint16_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint16_t>*>(::rust::RefMut<uint16_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint16_t> const*>(::rust::RefMut<uint16_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint16_t> const*>(::rust::RefMut<uint16_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint16_t> const*>(::rust::RefMut<uint16_t> const*&) {}

  template<>
  struct Ref< ::rust::RefMut<uint16_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<uint16_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<uint16_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<uint16_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<uint16_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<uint16_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<uint16_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::RefMut<uint16_t> > >(const ::rust::Ref< ::rust::RefMut<uint16_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint16_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<uint16_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<uint16_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<uint16_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<uint16_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<uint16_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<uint16_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::RefMut<uint16_t> > >(const ::rust::RefMut< ::rust::RefMut<uint16_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint16_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int32_t> >(const ::rust::Ref<int32_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int32_t> >(::rust::Ref<int32_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int32_t> >(::rust::Ref<int32_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::Ref<int32_t> >() {
    return sizeof(::rust::Ref<int32_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int32_t>*>(::rust::Ref<int32_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int32_t>*>(::rust::Ref<int32_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int32_t>*>(::rust::Ref<int32_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int32_t> const*>(::rust::Ref<int32_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int32_t> const*>(::rust::Ref<int32_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int32_t> const*>(::rust::Ref<int32_t> const*&) {}

  template<>
  struct Ref< ::rust::Ref<int32_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<int32_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<int32_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<int32_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<int32_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<int32_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<int32_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::Ref<int32_t> > >(const ::rust::Ref< ::rust::Ref<int32_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int32_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<int32_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<int32_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<int32_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<int32_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<int32_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<int32_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::Ref<int32_t> > >(const ::rust::RefMut< ::rust::Ref<int32_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int32_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint32_t> >(const ::rust::Ref<uint32_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint32_t> >(::rust::Ref<uint32_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint32_t> >(::rust::Ref<uint32_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::Ref<uint32_t> >() {
    return sizeof(::rust::Ref<uint32_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint32_t>*>(::rust::Ref<uint32_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint32_t>*>(::rust::Ref<uint32_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint32_t>*>(::rust::Ref<uint32_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint32_t> const*>(::rust::Ref<uint32_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint32_t> const*>(::rust::Ref<uint32_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint32_t> const*>(::rust::Ref<uint32_t> const*&) {}

  template<>
  struct Ref< ::rust::Ref<uint32_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<uint32_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<uint32_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<uint32_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<uint32_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<uint32_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<uint32_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::Ref<uint32_t> > >(const ::rust::Ref< ::rust::Ref<uint32_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint32_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<uint32_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<uint32_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<uint32_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<uint32_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<uint32_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<uint32_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::Ref<uint32_t> > >(const ::rust::RefMut< ::rust::Ref<uint32_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint32_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int32_t> >(const ::rust::RefMut<int32_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int32_t> >(::rust::RefMut<int32_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int32_t> >(::rust::RefMut<int32_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::RefMut<int32_t> >() {
    return sizeof(::rust::RefMut<int32_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int32_t>*>(::rust::RefMut<int32_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int32_t>*>(::rust::RefMut<int32_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int32_t>*>(::rust::RefMut<int32_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int32_t> const*>(::rust::RefMut<int32_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int32_t> const*>(::rust::RefMut<int32_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int32_t> const*>(::rust::RefMut<int32_t> const*&) {}

  template<>
  struct Ref< ::rust::RefMut<int32_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<int32_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<int32_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<int32_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<int32_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<int32_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<int32_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::RefMut<int32_t> > >(const ::rust::Ref< ::rust::RefMut<int32_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int32_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<int32_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<int32_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<int32_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<int32_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<int32_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<int32_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::RefMut<int32_t> > >(const ::rust::RefMut< ::rust::RefMut<int32_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int32_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint32_t> >(const ::rust::RefMut<uint32_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint32_t> >(::rust::RefMut<uint32_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint32_t> >(::rust::RefMut<uint32_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::RefMut<uint32_t> >() {
    return sizeof(::rust::RefMut<uint32_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint32_t>*>(::rust::RefMut<uint32_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint32_t>*>(::rust::RefMut<uint32_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint32_t>*>(::rust::RefMut<uint32_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint32_t> const*>(::rust::RefMut<uint32_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint32_t> const*>(::rust::RefMut<uint32_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint32_t> const*>(::rust::RefMut<uint32_t> const*&) {}

  template<>
  struct Ref< ::rust::RefMut<uint32_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<uint32_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<uint32_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<uint32_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<uint32_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<uint32_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<uint32_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::RefMut<uint32_t> > >(const ::rust::Ref< ::rust::RefMut<uint32_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint32_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<uint32_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<uint32_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<uint32_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<uint32_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<uint32_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<uint32_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::RefMut<uint32_t> > >(const ::rust::RefMut< ::rust::RefMut<uint32_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint32_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int64_t> >(const ::rust::Ref<int64_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int64_t> >(::rust::Ref<int64_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int64_t> >(::rust::Ref<int64_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::Ref<int64_t> >() {
    return sizeof(::rust::Ref<int64_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int64_t>*>(::rust::Ref<int64_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int64_t>*>(::rust::Ref<int64_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int64_t>*>(::rust::Ref<int64_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<int64_t> const*>(::rust::Ref<int64_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<int64_t> const*>(::rust::Ref<int64_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<int64_t> const*>(::rust::Ref<int64_t> const*&) {}

  template<>
  struct Ref< ::rust::Ref<int64_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<int64_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<int64_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<int64_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<int64_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<int64_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<int64_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::Ref<int64_t> > >(const ::rust::Ref< ::rust::Ref<int64_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int64_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<int64_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<int64_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<int64_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<int64_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<int64_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<int64_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::Ref<int64_t> > >(const ::rust::RefMut< ::rust::Ref<int64_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int64_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint64_t> >(const ::rust::Ref<uint64_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint64_t> >(::rust::Ref<uint64_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint64_t> >(::rust::Ref<uint64_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::Ref<uint64_t> >() {
    return sizeof(::rust::Ref<uint64_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint64_t>*>(::rust::Ref<uint64_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint64_t>*>(::rust::Ref<uint64_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint64_t>*>(::rust::Ref<uint64_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Ref<uint64_t> const*>(::rust::Ref<uint64_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::Ref<uint64_t> const*>(::rust::Ref<uint64_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::Ref<uint64_t> const*>(::rust::Ref<uint64_t> const*&) {}

  template<>
  struct Ref< ::rust::Ref<uint64_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<uint64_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<uint64_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<uint64_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<uint64_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<uint64_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<uint64_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::Ref<uint64_t> > >(const ::rust::Ref< ::rust::Ref<uint64_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint64_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<uint64_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<uint64_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<uint64_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<uint64_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<uint64_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<uint64_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::Ref<uint64_t> > >(const ::rust::RefMut< ::rust::Ref<uint64_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint64_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int64_t> >(const ::rust::RefMut<int64_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int64_t> >(::rust::RefMut<int64_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int64_t> >(::rust::RefMut<int64_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::RefMut<int64_t> >() {
    return sizeof(::rust::RefMut<int64_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int64_t>*>(::rust::RefMut<int64_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int64_t>*>(::rust::RefMut<int64_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int64_t>*>(::rust::RefMut<int64_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<int64_t> const*>(::rust::RefMut<int64_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<int64_t> const*>(::rust::RefMut<int64_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<int64_t> const*>(::rust::RefMut<int64_t> const*&) {}

  template<>
  struct Ref< ::rust::RefMut<int64_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<int64_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<int64_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<int64_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<int64_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<int64_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<int64_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::RefMut<int64_t> > >(const ::rust::Ref< ::rust::RefMut<int64_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int64_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<int64_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<int64_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<int64_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<int64_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<int64_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<int64_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::RefMut<int64_t> > >(const ::rust::RefMut< ::rust::RefMut<int64_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int64_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint64_t> >(const ::rust::RefMut<uint64_t>& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint64_t> >(::rust::RefMut<uint64_t>&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint64_t> >(::rust::RefMut<uint64_t>&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::RefMut<uint64_t> >() {
    return sizeof(::rust::RefMut<uint64_t>);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint64_t>*>(::rust::RefMut<uint64_t>* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint64_t>*>(::rust::RefMut<uint64_t>*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint64_t>*>(::rust::RefMut<uint64_t>*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::RefMut<uint64_t> const*>(::rust::RefMut<uint64_t> const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::RefMut<uint64_t> const*>(::rust::RefMut<uint64_t> const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::RefMut<uint64_t> const*>(::rust::RefMut<uint64_t> const*&) {}

  template<>
  struct Ref< ::rust::RefMut<uint64_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<uint64_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<uint64_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<uint64_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<uint64_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<uint64_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<uint64_t>*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::RefMut<uint64_t> > >(const ::rust::Ref< ::rust::RefMut<uint64_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint64_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<uint64_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<uint64_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<uint64_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<uint64_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<uint64_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<uint64_t>*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::RefMut<uint64_t> > >(const ::rust::RefMut< ::rust::RefMut<uint64_t> >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint64_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::ZngurCppOpaqueOwnedObject >(const ::rust::ZngurCppOpaqueOwnedObject& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::ZngurCppOpaqueOwnedObject >(::rust::ZngurCppOpaqueOwnedObject&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::ZngurCppOpaqueOwnedObject >(::rust::ZngurCppOpaqueOwnedObject&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::rust::ZngurCppOpaqueOwnedObject >() {
    return sizeof(::rust::ZngurCppOpaqueOwnedObject);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::ZngurCppOpaqueOwnedObject*>(::rust::ZngurCppOpaqueOwnedObject* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::ZngurCppOpaqueOwnedObject*>(::rust::ZngurCppOpaqueOwnedObject*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::ZngurCppOpaqueOwnedObject*>(::rust::ZngurCppOpaqueOwnedObject*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::ZngurCppOpaqueOwnedObject const*>(::rust::ZngurCppOpaqueOwnedObject const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::rust::ZngurCppOpaqueOwnedObject const*>(::rust::ZngurCppOpaqueOwnedObject const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::rust::ZngurCppOpaqueOwnedObject const*>(::rust::ZngurCppOpaqueOwnedObject const*&) {}

  template<>
  struct Ref< ::rust::ZngurCppOpaqueOwnedObject > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::ZngurCppOpaqueOwnedObject& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::ZngurCppOpaqueOwnedObject, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::ZngurCppOpaqueOwnedObject, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::ZngurCppOpaqueOwnedObject, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::ZngurCppOpaqueOwnedObject& operator*() {
      return *reinterpret_cast< ::rust::ZngurCppOpaqueOwnedObject*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::rust::ZngurCppOpaqueOwnedObject > >(const ::rust::Ref< ::rust::ZngurCppOpaqueOwnedObject >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::ZngurCppOpaqueOwnedObject > >;

  };

  template<>
  struct RefMut< ::rust::ZngurCppOpaqueOwnedObject > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::ZngurCppOpaqueOwnedObject& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::ZngurCppOpaqueOwnedObject, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::ZngurCppOpaqueOwnedObject, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::ZngurCppOpaqueOwnedObject& operator*() {
        return *reinterpret_cast< ::rust::ZngurCppOpaqueOwnedObject*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::rust::ZngurCppOpaqueOwnedObject > >(const ::rust::RefMut< ::rust::ZngurCppOpaqueOwnedObject >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::ZngurCppOpaqueOwnedObject > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::double_t >(const ::double_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::double_t >(::double_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::double_t >(::double_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::double_t >() {
    return sizeof(::double_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::double_t*>(::double_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::double_t*>(::double_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::double_t*>(::double_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::double_t const*>(::double_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::double_t const*>(::double_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::double_t const*>(::double_t const*&) {}

  template<>
  struct Ref< ::double_t > {
    Ref() {
      data = 0;
    }
    Ref(const ::double_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::double_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::double_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::double_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::double_t& operator*() {
      return *reinterpret_cast< ::double_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::double_t > >(const ::rust::Ref< ::double_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::double_t > >;

  };

  template<>
  struct RefMut< ::double_t > {
    RefMut() {
      data = 0;
    }

    RefMut(::double_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::double_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::double_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::double_t& operator*() {
        return *reinterpret_cast< ::double_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::double_t > >(const ::rust::RefMut< ::double_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::double_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< ::double_t > {
      static inline void print(::double_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::float_t >(const ::float_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::float_t >(::float_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::float_t >(::float_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::float_t >() {
    return sizeof(::float_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::float_t*>(::float_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::float_t*>(::float_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::float_t*>(::float_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::float_t const*>(::float_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::float_t const*>(::float_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::float_t const*>(::float_t const*&) {}

  template<>
  struct Ref< ::float_t > {
    Ref() {
      data = 0;
    }
    Ref(const ::float_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::float_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::float_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::float_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::float_t& operator*() {
      return *reinterpret_cast< ::float_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::float_t > >(const ::rust::Ref< ::float_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::float_t > >;

  };

  template<>
  struct RefMut< ::float_t > {
    RefMut() {
      data = 0;
    }

    RefMut(::float_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::float_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::float_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::float_t& operator*() {
        return *reinterpret_cast< ::float_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::float_t > >(const ::rust::RefMut< ::float_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::float_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< ::float_t > {
      static inline void print(::float_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  
    #if defined(__APPLE__) || defined(__wasm__)
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::size_t >(const ::size_t& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::size_t >(::size_t&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::size_t >(::size_t&) {}

  template<>
  inline size_t __zngur_internal_size_of< ::size_t >() {
    return sizeof(::size_t);
  }

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::size_t*>(::size_t* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::size_t*>(::size_t*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::size_t*>(::size_t*&) {}

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::size_t const*>(::size_t const* const & t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
  }

  template<>
  inline void __zngur_internal_assume_init< ::size_t const*>(::size_t const*&) {}
  template<>
  inline void __zngur_internal_assume_deinit< ::size_t const*>(::size_t const*&) {}

  template<>
  struct Ref< ::size_t > {
    Ref() {
      data = 0;
    }
    Ref(const ::size_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::size_t, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::size_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::size_t, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::size_t& operator*() {
      return *reinterpret_cast< ::size_t*>(data);
    }

  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref< ::size_t > >(const ::rust::Ref< ::size_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::size_t > >;

  };

  template<>
  struct RefMut< ::size_t > {
    RefMut() {
      data = 0;
    }

    RefMut(::size_t& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::size_t, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::size_t, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::size_t& operator*() {
        return *reinterpret_cast< ::size_t*>(data);
    }
  private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<RefMut< ::size_t > >(const ::rust::RefMut< ::size_t >& t) ;
    friend ::rust::ZngurPrettyPrinter< Ref< ::size_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< ::size_t > {
      static inline void print(::size_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  
    #endif
  

// end builtin types


} // namespace rust

extern "C" {
  
    void _zngur_crate_create_greeter_by_type_s12 (
      
        uint8_t* i0,
      
        uint8_t* i1,
      
        uint8_t* i2,
      
      uint8_t* o
    ) ;
  
    void _zngur_crate_create_person_s12 (
      
        uint8_t* i0,
      
      uint8_t* o
    ) ;
  
    void _zngur_crate_create_robot_s12 (
      
        uint8_t* i0,
      
      uint8_t* o
    ) ;
  
    void _zngur_crate_print_greeting_person_s12 (
      
        uint8_t* i0,
      
      uint8_t* o
    ) ;
  
    void _zngur_crate_print_greeting_robot_s12 (
      
        uint8_t* i0,
      
      uint8_t* o
    ) ;
  

  
    
      void _zngur__str_to_owned___x7n11m20y21 (
        
          uint8_t* i0,
        
        uint8_t* o
      ) ;
    

    

    

    

    
      
      
        
      
    

    
      
        void _zngur_crate_Greeter_s12(uint8_t *data, void destructor(uint8_t *), uint8_t *o);
        void _zngur_crate_Greeter_s12_borrowed(uint8_t *data, uint8_t *o);
      
    

  // end self.type_defs
  
    

    

    

    

    
      
      
        void _zngur__std_string_String_debug_pretty_s7s11s18e25(uint8_t*);
        void _zngur__std_string_String_debug_print_s7s11s18e25(uint8_t*);
      
    
      
      
        void _zngur__std_string_String_drop_in_place_s7s11s18e25(uint8_t*);
      
    

    
      
        void _zngur_crate_Greeter_s12(uint8_t *data, void destructor(uint8_t *), uint8_t *o);
        void _zngur_crate_Greeter_s12_borrowed(uint8_t *data, uint8_t *o);
      
    

  // end self.type_defs
  
    

    
      void _zngur_crate_Person_s12 (
        
          uint8_t* i0,
        
        uint8_t* o
      ) ;
    

    

    

    
      
      
        void _zngur_crate_Person_debug_pretty_s12e19(uint8_t*);
        void _zngur_crate_Person_debug_print_s12e19(uint8_t*);
      
    
      
      
        void _zngur_crate_Person_drop_in_place_s12e19(uint8_t*);
      
    

    
      
        void _zngur_crate_Greeter_s12(uint8_t *data, void destructor(uint8_t *), uint8_t *o);
        void _zngur_crate_Greeter_s12_borrowed(uint8_t *data, uint8_t *o);
      
    

  // end self.type_defs
  
    

    
      void _zngur_crate_Robot_s12 (
        
          uint8_t* i0,
        
        uint8_t* o
      ) ;
    

    

    

    
      
      
        void _zngur_crate_Robot_debug_pretty_s12e18(uint8_t*);
        void _zngur_crate_Robot_debug_print_s12e18(uint8_t*);
      
    
      
      
        void _zngur_crate_Robot_drop_in_place_s12e18(uint8_t*);
      
    

    
      
        void _zngur_crate_Greeter_s12(uint8_t *data, void destructor(uint8_t *), uint8_t *o);
        void _zngur_crate_Greeter_s12_borrowed(uint8_t *data, uint8_t *o);
      
    

  // end self.type_defs
  
    
      void _zngur__Box_dyncrate_Greeter__greet___x7x11s20y28n29m35y36 (
        
          uint8_t* i0,
        
        uint8_t* o
      ) ;
    

    

    

    

    
      
      
        void _zngur_Box_dyncrate_Greeter__drop_in_place_x10s19y27e28(uint8_t*);
      
    

    
      
        void _zngur_crate_Greeter_s12(uint8_t *data, void destructor(uint8_t *), uint8_t *o);
        void _zngur_crate_Greeter_s12_borrowed(uint8_t *data, uint8_t *o);
      
    

  // end self.type_defs
  
    

    

    

    

    
      
      
        
      
    

    
      
        void _zngur_crate_Greeter_s12(uint8_t *data, void destructor(uint8_t *), uint8_t *o);
        void _zngur_crate_Greeter_s12_borrowed(uint8_t *data, uint8_t *o);
      
    

  // end self.type_defs
  

} // extern "C"


  namespace rust {
struct Str;
}


  namespace rust {
namespace std {
namespace string {
struct String;
}
}
}


  namespace rust {
namespace crate {
struct Person;
}
}


  namespace rust {
namespace crate {
struct Robot;
}
}


  namespace rust {
namespace crate {
struct Greeter;
}
}
namespace rust {
template<typename ...T>
struct Dyn;
}
namespace rust {
template<typename ...T>
struct Box;
}


  




namespace rust {


        template<>
        struct zngur_is_unsized< ::rust::Str > : ::std::true_type {};


}


  
    namespace rust {
    namespace crate {
    struct Greeter {
      public:
        virtual ~Greeter() {};
        
          virtual ::rust::std::string::String greet (
            
          ) = 0;
        
      };
    } // namespace rust
    } // namespace crate
  



  
  
  

  namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr< ::rust::Str >(const ::rust::Str& t) ;
    template<>
    inline void __zngur_internal_check_init< ::rust::Str >(const ::rust::Str& t) ;
    template<>
    inline void __zngur_internal_assume_init< ::rust::Str >(::rust::Str& t) ;
    template<>
    inline void __zngur_internal_assume_deinit< ::rust::Str >(::rust::Str& t) ;
    template<>
    inline size_t __zngur_internal_size_of< ::rust::Str >() ;
  }

  namespace rust {

    
      struct Str {
      
        public:
          Str() = delete;
      

      
      

      
          static ::rust::std::string::String to_owned(
            ::rust::Ref< ::rust::Str > i0
          ) ;
          
              ::rust::std::string::String to_owned(
                
              )
               const  ;
          
      

      

      

    }; // struct Str

    // end !rust unit
    

} // namespace rust

namespace rust {


  




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::Str > {
  private:
     ::std::array<size_t, 2>  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::RefMut< ::rust::Str > >(const ::rust::RefMut< ::rust::Str >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::Str > >;
  public:
    RefMut() {
      data =  {0, 0} ;
    }

    friend Ref< ::rust::Str >;

    

    

    

    
      

    

    

    
      
        ::rust::std::string::String to_owned(
          
        ) const ;
      
    

  }; // struct RefMut< ::rust::Str >

  template<>
  inline uint8_t* __zngur_internal_data_ptr< RefMut < ::rust::Str > >(const RefMut< ::rust::Str >& t) {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }

  template<>
  inline void __zngur_internal_assume_init< RefMut < ::rust::Str > >(RefMut< ::rust::Str >&) {}

  template<>
  inline void __zngur_internal_check_init< RefMut < ::rust::Str > >(const RefMut< ::rust::Str >&) {}

  template<>
  inline void __zngur_internal_assume_deinit< RefMut < ::rust::Str > >(RefMut< ::rust::Str >&) {}

  template<>
  inline size_t __zngur_internal_size_of< RefMut < ::rust::Str > >() {
      return 16;
  }

} // namespace rust

// Ref specialization

    auto operator""_rs(const char* input, size_t len) -> ::rust::Ref<::rust::Str>;


namespace rust {

  template<>
  struct Ref< ::rust::Str > {
  private:
     ::std::array<size_t, 2>  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Ref< ::rust::Str > >(const ::rust::Ref< ::rust::Str >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::Str > >;
  public:
    Ref() {
      data =  {0, 0} ;
    }

    

      Ref(RefMut< ::rust::Str > rm) {
          data = rm.data;
      }

    

    
    

    

    

    
      
        
          ::rust::std::string::String to_owned() const ;
        
      
    

    
      friend auto ::operator""_rs(const char* input, size_t len) -> ::rust::Ref<::rust::Str>;
    
};



template<>
inline uint8_t* __zngur_internal_data_ptr< Ref < ::rust::Str > >(const Ref < ::rust::Str >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Ref < ::rust::Str > >(Ref < ::rust::Str >&) {}

template<>
inline void __zngur_internal_check_init< Ref < ::rust::Str > >(const Ref < ::rust::Str >&) {}

template<>
inline void __zngur_internal_assume_deinit< Ref < ::rust::Str > >(Ref < ::rust::Str >&) {}

template<>
inline size_t __zngur_internal_size_of< Ref < ::rust::Str > >() {
    return 16;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< Raw < ::rust::Str > >(const Raw < ::rust::Str >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Raw < ::rust::Str > >(Raw < ::rust::Str >&) {}

template<>
inline void __zngur_internal_check_init< Raw < ::rust::Str > >(const Raw < ::rust::Str >&) {}

template<>
inline void __zngur_internal_assume_deinit< Raw < ::rust::Str > >(Raw < ::rust::Str >&) {}

template<>
inline size_t __zngur_internal_size_of< Raw < ::rust::Str > >() {
    return 16;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< RawMut < ::rust::Str > >(const RawMut < ::rust::Str >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< RawMut < ::rust::Str > >(RawMut < ::rust::Str >&) {}

template<>
inline void __zngur_internal_check_init< RawMut < ::rust::Str > >(const RawMut < ::rust::Str >&) {}

template<>
inline void __zngur_internal_assume_deinit< RawMut < ::rust::Str > >(RawMut < ::rust::Str >&) {}

template<>
inline size_t __zngur_internal_size_of< RawMut < ::rust::Str > >() {
    return 16;
}



} // namespace rust


  inline ::rust::Ref<::rust::Str> operator""_rs(const char* input, size_t len) {
    ::rust::Ref<::rust::Str> o;
    o.data[0] = reinterpret_cast<size_t>(input);
    o.data[1] = len;
    return o;
  }


namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::Str, OFFSET > {

    

    
      
        
          ::rust::std::string::String to_owned(
            
          ) const ;
        
      
    

  }; // struct FieldOwned< ::rust::Str, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::Str, OFFSET > {

    

    
      
        
          ::rust::std::string::String to_owned(
            
          ) const ;
        
      
    

  }; // struct FieldRef< ::rust::Str, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::Str, OFFSET > {

    

    
      
        
          ::rust::std::string::String to_owned(
            
          ) const ;
        
      
    

  }; // struct FieldRefMut< ::rust::Str, OFFSET >


} // namespace rust


  
  
  

  namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr< ::rust::std::string::String >(const ::rust::std::string::String& t) ;
    template<>
    inline void __zngur_internal_check_init< ::rust::std::string::String >(const ::rust::std::string::String& t) ;
    template<>
    inline void __zngur_internal_assume_init< ::rust::std::string::String >(::rust::std::string::String& t) ;
    template<>
    inline void __zngur_internal_assume_deinit< ::rust::std::string::String >(::rust::std::string::String& t) ;
    template<>
    inline size_t __zngur_internal_size_of< ::rust::std::string::String >() ;
  }

  namespace rust {
    namespace std {
        namespace string {

    
      struct String {
      
        private:
          alignas(8) mutable ::std::array< ::uint8_t, 24> data;
      

      
        friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::std::string::String >(const ::rust::std::string::String& t) ;
        friend void ::rust::__zngur_internal_check_init< ::rust::std::string::String >(const ::rust::std::string::String& t) ;
        friend void ::rust::__zngur_internal_assume_init< ::rust::std::string::String >(::rust::std::string::String& t) ;
        friend void ::rust::__zngur_internal_assume_deinit< ::rust::std::string::String >(::rust::std::string::String& t) ;
        friend ::rust::ZngurPrettyPrinter< ::rust::std::string::String >;

      

      
        bool drop_flag;
      

      

      public:
        
          

          String() : drop_flag(false) {  }
          ~String() {
            if (drop_flag) {
              _zngur__std_string_String_drop_in_place_s7s11s18e25(&data[0]);
            }
            
          }
          String(const String& other) = delete;
          String& operator=(const String& other) = delete;
          String(String&& other) : drop_flag(false) {
            
            *this = ::std::move(other);
          }
          String& operator=(String&& other) {
            if (this != &other) {
              if (drop_flag) {
                _zngur__std_string_String_drop_in_place_s7s11s18e25(&data[0]);
              }
              this->drop_flag = other.drop_flag;
              this->data = other.data;
              other.drop_flag = false;
            }
            return *this;
          }
        

        
        

        

      

      

      

      

    }; // struct String

    // end !rust unit
    

} // namespace rust
    } // namespace std
        } // namespace string

namespace rust {


  template<>
  inline size_t __zngur_internal_size_of< ::rust::std::string::String >() {
      return 24;
  }



  
    template<>
    inline void __zngur_internal_check_init< ::rust::std::string::String >(const ::rust::std::string::String& t) {
        if (!t.drop_flag) {
            ::std::cerr << "Use of uninitialized or moved Zngur Rust object with type ::rust::std::string::String" << ::std::endl;
            while (true) raise(SIGSEGV);
        }
    }

    template<>
    inline void __zngur_internal_assume_init< ::rust::std::string::String >(::rust::std::string::String& t) {
        t.drop_flag = true;
    }

    template<>
    inline void __zngur_internal_assume_deinit< ::rust::std::string::String >(::rust::std::string::String& t) {
        ::rust::__zngur_internal_check_init< ::rust::std::string::String >(t);
        t.drop_flag = false;
    }
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::std::string::String >(::rust::std::string::String const & t) {
      return const_cast<uint8_t*>(&t.data[0]);
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::std::string::String > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::RefMut< ::rust::std::string::String > >(const ::rust::RefMut< ::rust::std::string::String >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::std::string::String > >;
  public:
    RefMut() {
      data =  0 ;
    }

    friend Ref< ::rust::std::string::String >;

    
      
    

    
      RefMut(const ::rust::std::string::String& t) {
        ::rust::__zngur_internal_check_init< ::rust::std::string::String >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }
    

    
      template<size_t OFFSET>
      RefMut(const FieldOwned< ::rust::std::string::String, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }
      template<size_t OFFSET>
      RefMut(const FieldRefMut< ::rust::std::string::String, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
      

    

    

    

  }; // struct RefMut< ::rust::std::string::String >

  template<>
  inline uint8_t* __zngur_internal_data_ptr< RefMut < ::rust::std::string::String > >(const RefMut< ::rust::std::string::String >& t) {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }

  template<>
  inline void __zngur_internal_assume_init< RefMut < ::rust::std::string::String > >(RefMut< ::rust::std::string::String >&) {}

  template<>
  inline void __zngur_internal_check_init< RefMut < ::rust::std::string::String > >(const RefMut< ::rust::std::string::String >&) {}

  template<>
  inline void __zngur_internal_assume_deinit< RefMut < ::rust::std::string::String > >(RefMut< ::rust::std::string::String >&) {}

  template<>
  inline size_t __zngur_internal_size_of< RefMut < ::rust::std::string::String > >() {
      return 8;
  }

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::std::string::String > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Ref< ::rust::std::string::String > >(const ::rust::Ref< ::rust::std::string::String >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::std::string::String > >;
  public:
    Ref() {
      data =  0 ;
    }

    
      Ref(const ::rust::std::string::String& t) {
        ::rust::__zngur_internal_check_init< ::rust::std::string::String >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }

      

    

      Ref(RefMut< ::rust::std::string::String > rm) {
          data = rm.data;
      }

    
      template<size_t OFFSET>
      Ref(const FieldOwned< ::rust::std::string::String, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRef< ::rust::std::string::String, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRefMut< ::rust::std::string::String, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
    

    

    

    

    
};



template<>
inline uint8_t* __zngur_internal_data_ptr< Ref < ::rust::std::string::String > >(const Ref < ::rust::std::string::String >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Ref < ::rust::std::string::String > >(Ref < ::rust::std::string::String >&) {}

template<>
inline void __zngur_internal_check_init< Ref < ::rust::std::string::String > >(const Ref < ::rust::std::string::String >&) {}

template<>
inline void __zngur_internal_assume_deinit< Ref < ::rust::std::string::String > >(Ref < ::rust::std::string::String >&) {}

template<>
inline size_t __zngur_internal_size_of< Ref < ::rust::std::string::String > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< Raw < ::rust::std::string::String > >(const Raw < ::rust::std::string::String >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Raw < ::rust::std::string::String > >(Raw < ::rust::std::string::String >&) {}

template<>
inline void __zngur_internal_check_init< Raw < ::rust::std::string::String > >(const Raw < ::rust::std::string::String >&) {}

template<>
inline void __zngur_internal_assume_deinit< Raw < ::rust::std::string::String > >(Raw < ::rust::std::string::String >&) {}

template<>
inline size_t __zngur_internal_size_of< Raw < ::rust::std::string::String > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< RawMut < ::rust::std::string::String > >(const RawMut < ::rust::std::string::String >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< RawMut < ::rust::std::string::String > >(RawMut < ::rust::std::string::String >&) {}

template<>
inline void __zngur_internal_check_init< RawMut < ::rust::std::string::String > >(const RawMut < ::rust::std::string::String >&) {}

template<>
inline void __zngur_internal_assume_deinit< RawMut < ::rust::std::string::String > >(RawMut < ::rust::std::string::String >&) {}

template<>
inline size_t __zngur_internal_size_of< RawMut < ::rust::std::string::String > >() {
    return 8;
}



} // namespace rust



namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::std::string::String, OFFSET > {

    

    

  }; // struct FieldOwned< ::rust::std::string::String, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::std::string::String, OFFSET > {

    

    

  }; // struct FieldRef< ::rust::std::string::String, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::std::string::String, OFFSET > {

    

    

  }; // struct FieldRefMut< ::rust::std::string::String, OFFSET >


} // namespace rust


  
  
  

  namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr< ::rust::crate::Person >(const ::rust::crate::Person& t) ;
    template<>
    inline void __zngur_internal_check_init< ::rust::crate::Person >(const ::rust::crate::Person& t) ;
    template<>
    inline void __zngur_internal_assume_init< ::rust::crate::Person >(::rust::crate::Person& t) ;
    template<>
    inline void __zngur_internal_assume_deinit< ::rust::crate::Person >(::rust::crate::Person& t) ;
    template<>
    inline size_t __zngur_internal_size_of< ::rust::crate::Person >() ;
  }

  namespace rust {
    namespace crate {

    
      struct Person {
      
        private:
          alignas(8) mutable ::std::array< ::uint8_t, 24> data;
      

      
        friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::crate::Person >(const ::rust::crate::Person& t) ;
        friend void ::rust::__zngur_internal_check_init< ::rust::crate::Person >(const ::rust::crate::Person& t) ;
        friend void ::rust::__zngur_internal_assume_init< ::rust::crate::Person >(::rust::crate::Person& t) ;
        friend void ::rust::__zngur_internal_assume_deinit< ::rust::crate::Person >(::rust::crate::Person& t) ;
        friend ::rust::ZngurPrettyPrinter< ::rust::crate::Person >;

      

      
        bool drop_flag;
      

      

      public:
        
          

          Person() : drop_flag(false) {  }
          ~Person() {
            if (drop_flag) {
              _zngur_crate_Person_drop_in_place_s12e19(&data[0]);
            }
            
          }
          Person(const Person& other) = delete;
          Person& operator=(const Person& other) = delete;
          Person(Person&& other) : drop_flag(false) {
            
            *this = ::std::move(other);
          }
          Person& operator=(Person&& other) {
            if (this != &other) {
              if (drop_flag) {
                _zngur_crate_Person_drop_in_place_s12e19(&data[0]);
              }
              this->drop_flag = other.drop_flag;
              this->data = other.data;
              other.drop_flag = false;
            }
            return *this;
          }
        

        
        

        

      

      

      
        Person(
          ::rust::std::string::String i0
        ) ;
      

      

    }; // struct Person

    // end !rust unit
    

} // namespace rust
    } // namespace crate

namespace rust {


  template<>
  inline size_t __zngur_internal_size_of< ::rust::crate::Person >() {
      return 24;
  }



  
    template<>
    inline void __zngur_internal_check_init< ::rust::crate::Person >(const ::rust::crate::Person& t) {
        if (!t.drop_flag) {
            ::std::cerr << "Use of uninitialized or moved Zngur Rust object with type ::rust::crate::Person" << ::std::endl;
            while (true) raise(SIGSEGV);
        }
    }

    template<>
    inline void __zngur_internal_assume_init< ::rust::crate::Person >(::rust::crate::Person& t) {
        t.drop_flag = true;
    }

    template<>
    inline void __zngur_internal_assume_deinit< ::rust::crate::Person >(::rust::crate::Person& t) {
        ::rust::__zngur_internal_check_init< ::rust::crate::Person >(t);
        t.drop_flag = false;
    }
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::crate::Person >(::rust::crate::Person const & t) {
      return const_cast<uint8_t*>(&t.data[0]);
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::crate::Person > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::RefMut< ::rust::crate::Person > >(const ::rust::RefMut< ::rust::crate::Person >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::crate::Person > >;
  public:
    RefMut() {
      data =  0 ;
    }

    friend Ref< ::rust::crate::Person >;

    
      
    

    
      RefMut(const ::rust::crate::Person& t) {
        ::rust::__zngur_internal_check_init< ::rust::crate::Person >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }
    

    
      template<size_t OFFSET>
      RefMut(const FieldOwned< ::rust::crate::Person, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }
      template<size_t OFFSET>
      RefMut(const FieldRefMut< ::rust::crate::Person, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
      

    

    

    

  }; // struct RefMut< ::rust::crate::Person >

  template<>
  inline uint8_t* __zngur_internal_data_ptr< RefMut < ::rust::crate::Person > >(const RefMut< ::rust::crate::Person >& t) {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }

  template<>
  inline void __zngur_internal_assume_init< RefMut < ::rust::crate::Person > >(RefMut< ::rust::crate::Person >&) {}

  template<>
  inline void __zngur_internal_check_init< RefMut < ::rust::crate::Person > >(const RefMut< ::rust::crate::Person >&) {}

  template<>
  inline void __zngur_internal_assume_deinit< RefMut < ::rust::crate::Person > >(RefMut< ::rust::crate::Person >&) {}

  template<>
  inline size_t __zngur_internal_size_of< RefMut < ::rust::crate::Person > >() {
      return 8;
  }

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::crate::Person > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Ref< ::rust::crate::Person > >(const ::rust::Ref< ::rust::crate::Person >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::crate::Person > >;
  public:
    Ref() {
      data =  0 ;
    }

    
      Ref(const ::rust::crate::Person& t) {
        ::rust::__zngur_internal_check_init< ::rust::crate::Person >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }

      

    

      Ref(RefMut< ::rust::crate::Person > rm) {
          data = rm.data;
      }

    
      template<size_t OFFSET>
      Ref(const FieldOwned< ::rust::crate::Person, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRef< ::rust::crate::Person, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRefMut< ::rust::crate::Person, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
    

    

    

    

    
};



template<>
inline uint8_t* __zngur_internal_data_ptr< Ref < ::rust::crate::Person > >(const Ref < ::rust::crate::Person >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Ref < ::rust::crate::Person > >(Ref < ::rust::crate::Person >&) {}

template<>
inline void __zngur_internal_check_init< Ref < ::rust::crate::Person > >(const Ref < ::rust::crate::Person >&) {}

template<>
inline void __zngur_internal_assume_deinit< Ref < ::rust::crate::Person > >(Ref < ::rust::crate::Person >&) {}

template<>
inline size_t __zngur_internal_size_of< Ref < ::rust::crate::Person > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< Raw < ::rust::crate::Person > >(const Raw < ::rust::crate::Person >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Raw < ::rust::crate::Person > >(Raw < ::rust::crate::Person >&) {}

template<>
inline void __zngur_internal_check_init< Raw < ::rust::crate::Person > >(const Raw < ::rust::crate::Person >&) {}

template<>
inline void __zngur_internal_assume_deinit< Raw < ::rust::crate::Person > >(Raw < ::rust::crate::Person >&) {}

template<>
inline size_t __zngur_internal_size_of< Raw < ::rust::crate::Person > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< RawMut < ::rust::crate::Person > >(const RawMut < ::rust::crate::Person >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< RawMut < ::rust::crate::Person > >(RawMut < ::rust::crate::Person >&) {}

template<>
inline void __zngur_internal_check_init< RawMut < ::rust::crate::Person > >(const RawMut < ::rust::crate::Person >&) {}

template<>
inline void __zngur_internal_assume_deinit< RawMut < ::rust::crate::Person > >(RawMut < ::rust::crate::Person >&) {}

template<>
inline size_t __zngur_internal_size_of< RawMut < ::rust::crate::Person > >() {
    return 8;
}



} // namespace rust



namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::crate::Person, OFFSET > {

    

    

  }; // struct FieldOwned< ::rust::crate::Person, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::crate::Person, OFFSET > {

    

    

  }; // struct FieldRef< ::rust::crate::Person, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::crate::Person, OFFSET > {

    

    

  }; // struct FieldRefMut< ::rust::crate::Person, OFFSET >


} // namespace rust


  
  
  

  namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr< ::rust::crate::Robot >(const ::rust::crate::Robot& t) ;
    template<>
    inline void __zngur_internal_check_init< ::rust::crate::Robot >(const ::rust::crate::Robot& t) ;
    template<>
    inline void __zngur_internal_assume_init< ::rust::crate::Robot >(::rust::crate::Robot& t) ;
    template<>
    inline void __zngur_internal_assume_deinit< ::rust::crate::Robot >(::rust::crate::Robot& t) ;
    template<>
    inline size_t __zngur_internal_size_of< ::rust::crate::Robot >() ;
  }

  namespace rust {
    namespace crate {

    
      struct Robot {
      
        private:
          alignas(4) mutable ::std::array< ::uint8_t, 4> data;
      

      
        friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::crate::Robot >(const ::rust::crate::Robot& t) ;
        friend void ::rust::__zngur_internal_check_init< ::rust::crate::Robot >(const ::rust::crate::Robot& t) ;
        friend void ::rust::__zngur_internal_assume_init< ::rust::crate::Robot >(::rust::crate::Robot& t) ;
        friend void ::rust::__zngur_internal_assume_deinit< ::rust::crate::Robot >(::rust::crate::Robot& t) ;
        friend ::rust::ZngurPrettyPrinter< ::rust::crate::Robot >;

      

      
        bool drop_flag;
      

      

      public:
        
          

          Robot() : drop_flag(false) {  }
          ~Robot() {
            if (drop_flag) {
              _zngur_crate_Robot_drop_in_place_s12e18(&data[0]);
            }
            
          }
          Robot(const Robot& other) = delete;
          Robot& operator=(const Robot& other) = delete;
          Robot(Robot&& other) : drop_flag(false) {
            
            *this = ::std::move(other);
          }
          Robot& operator=(Robot&& other) {
            if (this != &other) {
              if (drop_flag) {
                _zngur_crate_Robot_drop_in_place_s12e18(&data[0]);
              }
              this->drop_flag = other.drop_flag;
              this->data = other.data;
              other.drop_flag = false;
            }
            return *this;
          }
        

        
        

        

      

      

      
        Robot(
          ::uint32_t i0
        ) ;
      

      

    }; // struct Robot

    // end !rust unit
    

} // namespace rust
    } // namespace crate

namespace rust {


  template<>
  inline size_t __zngur_internal_size_of< ::rust::crate::Robot >() {
      return 4;
  }



  
    template<>
    inline void __zngur_internal_check_init< ::rust::crate::Robot >(const ::rust::crate::Robot& t) {
        if (!t.drop_flag) {
            ::std::cerr << "Use of uninitialized or moved Zngur Rust object with type ::rust::crate::Robot" << ::std::endl;
            while (true) raise(SIGSEGV);
        }
    }

    template<>
    inline void __zngur_internal_assume_init< ::rust::crate::Robot >(::rust::crate::Robot& t) {
        t.drop_flag = true;
    }

    template<>
    inline void __zngur_internal_assume_deinit< ::rust::crate::Robot >(::rust::crate::Robot& t) {
        ::rust::__zngur_internal_check_init< ::rust::crate::Robot >(t);
        t.drop_flag = false;
    }
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::crate::Robot >(::rust::crate::Robot const & t) {
      return const_cast<uint8_t*>(&t.data[0]);
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::crate::Robot > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::RefMut< ::rust::crate::Robot > >(const ::rust::RefMut< ::rust::crate::Robot >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::crate::Robot > >;
  public:
    RefMut() {
      data =  0 ;
    }

    friend Ref< ::rust::crate::Robot >;

    
      
    

    
      RefMut(const ::rust::crate::Robot& t) {
        ::rust::__zngur_internal_check_init< ::rust::crate::Robot >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }
    

    
      template<size_t OFFSET>
      RefMut(const FieldOwned< ::rust::crate::Robot, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }
      template<size_t OFFSET>
      RefMut(const FieldRefMut< ::rust::crate::Robot, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
      

    

    

    

  }; // struct RefMut< ::rust::crate::Robot >

  template<>
  inline uint8_t* __zngur_internal_data_ptr< RefMut < ::rust::crate::Robot > >(const RefMut< ::rust::crate::Robot >& t) {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }

  template<>
  inline void __zngur_internal_assume_init< RefMut < ::rust::crate::Robot > >(RefMut< ::rust::crate::Robot >&) {}

  template<>
  inline void __zngur_internal_check_init< RefMut < ::rust::crate::Robot > >(const RefMut< ::rust::crate::Robot >&) {}

  template<>
  inline void __zngur_internal_assume_deinit< RefMut < ::rust::crate::Robot > >(RefMut< ::rust::crate::Robot >&) {}

  template<>
  inline size_t __zngur_internal_size_of< RefMut < ::rust::crate::Robot > >() {
      return 8;
  }

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::crate::Robot > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Ref< ::rust::crate::Robot > >(const ::rust::Ref< ::rust::crate::Robot >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::crate::Robot > >;
  public:
    Ref() {
      data =  0 ;
    }

    
      Ref(const ::rust::crate::Robot& t) {
        ::rust::__zngur_internal_check_init< ::rust::crate::Robot >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }

      

    

      Ref(RefMut< ::rust::crate::Robot > rm) {
          data = rm.data;
      }

    
      template<size_t OFFSET>
      Ref(const FieldOwned< ::rust::crate::Robot, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRef< ::rust::crate::Robot, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRefMut< ::rust::crate::Robot, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
    

    

    

    

    
};



template<>
inline uint8_t* __zngur_internal_data_ptr< Ref < ::rust::crate::Robot > >(const Ref < ::rust::crate::Robot >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Ref < ::rust::crate::Robot > >(Ref < ::rust::crate::Robot >&) {}

template<>
inline void __zngur_internal_check_init< Ref < ::rust::crate::Robot > >(const Ref < ::rust::crate::Robot >&) {}

template<>
inline void __zngur_internal_assume_deinit< Ref < ::rust::crate::Robot > >(Ref < ::rust::crate::Robot >&) {}

template<>
inline size_t __zngur_internal_size_of< Ref < ::rust::crate::Robot > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< Raw < ::rust::crate::Robot > >(const Raw < ::rust::crate::Robot >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Raw < ::rust::crate::Robot > >(Raw < ::rust::crate::Robot >&) {}

template<>
inline void __zngur_internal_check_init< Raw < ::rust::crate::Robot > >(const Raw < ::rust::crate::Robot >&) {}

template<>
inline void __zngur_internal_assume_deinit< Raw < ::rust::crate::Robot > >(Raw < ::rust::crate::Robot >&) {}

template<>
inline size_t __zngur_internal_size_of< Raw < ::rust::crate::Robot > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< RawMut < ::rust::crate::Robot > >(const RawMut < ::rust::crate::Robot >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< RawMut < ::rust::crate::Robot > >(RawMut < ::rust::crate::Robot >&) {}

template<>
inline void __zngur_internal_check_init< RawMut < ::rust::crate::Robot > >(const RawMut < ::rust::crate::Robot >&) {}

template<>
inline void __zngur_internal_assume_deinit< RawMut < ::rust::crate::Robot > >(RawMut < ::rust::crate::Robot >&) {}

template<>
inline size_t __zngur_internal_size_of< RawMut < ::rust::crate::Robot > >() {
    return 8;
}



} // namespace rust



namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::crate::Robot, OFFSET > {

    

    

  }; // struct FieldOwned< ::rust::crate::Robot, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::crate::Robot, OFFSET > {

    

    

  }; // struct FieldRef< ::rust::crate::Robot, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::crate::Robot, OFFSET > {

    

    

  }; // struct FieldRefMut< ::rust::crate::Robot, OFFSET >


} // namespace rust


  
  
  

  namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(const ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) ;
    template<>
    inline void __zngur_internal_check_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(const ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) ;
    template<>
    inline void __zngur_internal_assume_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) ;
    template<>
    inline void __zngur_internal_assume_deinit< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) ;
    template<>
    inline size_t __zngur_internal_size_of< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >() ;
  }

  namespace rust {

    
      template<> struct Box< ::rust::Dyn< ::rust::crate::Greeter > > {
      
        private:
          alignas(8) mutable ::std::array< ::uint8_t, 16> data;
      

      
        friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(const ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) ;
        friend void ::rust::__zngur_internal_check_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(const ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) ;
        friend void ::rust::__zngur_internal_assume_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) ;
        friend void ::rust::__zngur_internal_assume_deinit< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) ;
        friend ::rust::ZngurPrettyPrinter< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >;

      

      
        bool drop_flag;
      

      

      public:
        
          

          Box() : drop_flag(false) {  }
          ~Box() {
            if (drop_flag) {
              _zngur_Box_dyncrate_Greeter__drop_in_place_x10s19y27e28(&data[0]);
            }
            
          }
          Box(const Box& other) = delete;
          Box& operator=(const Box& other) = delete;
          Box(Box&& other) : drop_flag(false) {
            
            *this = ::std::move(other);
          }
          Box& operator=(Box&& other) {
            if (this != &other) {
              if (drop_flag) {
                _zngur_Box_dyncrate_Greeter__drop_in_place_x10s19y27e28(&data[0]);
              }
              this->drop_flag = other.drop_flag;
              this->data = other.data;
              other.drop_flag = false;
            }
            return *this;
          }
        

        
          template<typename T, typename... Args>
          static inline Box make_box(Args&&... args);
        

        

      

      
          static ::rust::std::string::String greet(
            ::rust::Ref< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > i0
          ) ;
          
              ::rust::std::string::String greet(
                
              )
               const  ;
          
      

      

      

    }; // template<> struct Box< ::rust::Dyn< ::rust::crate::Greeter > >

    // end !rust unit
    

} // namespace rust

namespace rust {


  template<>
  inline size_t __zngur_internal_size_of< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >() {
      return 16;
  }



  
    template<>
    inline void __zngur_internal_check_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(const ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) {
        if (!t.drop_flag) {
            ::std::cerr << "Use of uninitialized or moved Zngur Rust object with type ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >" << ::std::endl;
            while (true) raise(SIGSEGV);
        }
    }

    template<>
    inline void __zngur_internal_assume_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) {
        t.drop_flag = true;
    }

    template<>
    inline void __zngur_internal_assume_deinit< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) {
        ::rust::__zngur_internal_check_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(t);
        t.drop_flag = false;
    }
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > const & t) {
      return const_cast<uint8_t*>(&t.data[0]);
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const ::rust::RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >;
  public:
    RefMut() {
      data =  0 ;
    }

    friend Ref< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >;

    
      
    

    
      RefMut(const ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) {
        ::rust::__zngur_internal_check_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }
    

    
      template<size_t OFFSET>
      RefMut(const FieldOwned< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }
      template<size_t OFFSET>
      RefMut(const FieldRefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
      

    

    

    
      
        ::rust::std::string::String greet(
          
        ) const ;
      
    

  }; // struct RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >

  template<>
  inline uint8_t* __zngur_internal_data_ptr< RefMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >& t) {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }

  template<>
  inline void __zngur_internal_assume_init< RefMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

  template<>
  inline void __zngur_internal_check_init< RefMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

  template<>
  inline void __zngur_internal_assume_deinit< RefMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

  template<>
  inline size_t __zngur_internal_size_of< RefMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >() {
      return 8;
  }

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Ref< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const ::rust::Ref< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >;
  public:
    Ref() {
      data =  0 ;
    }

    
      Ref(const ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >& t) {
        ::rust::__zngur_internal_check_init< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }

      

    

      Ref(RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > rm) {
          data = rm.data;
      }

    
      template<size_t OFFSET>
      Ref(const FieldOwned< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRef< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
    

    

    

    
      
        
          ::rust::std::string::String greet() const ;
        
      
    

    
};



template<>
inline uint8_t* __zngur_internal_data_ptr< Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline void __zngur_internal_check_init< Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline void __zngur_internal_assume_deinit< Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline size_t __zngur_internal_size_of< Ref < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline void __zngur_internal_check_init< Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline void __zngur_internal_assume_deinit< Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline size_t __zngur_internal_size_of< Raw < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline void __zngur_internal_check_init< RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(const RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline void __zngur_internal_assume_deinit< RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >(RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >&) {}

template<>
inline size_t __zngur_internal_size_of< RawMut < ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > >() {
    return 8;
}



} // namespace rust



namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET > {

    

    
      
        
          ::rust::std::string::String greet(
            
          ) const ;
        
      
    

  }; // struct FieldOwned< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET > {

    

    
      
        
          ::rust::std::string::String greet(
            
          ) const ;
        
      
    

  }; // struct FieldRef< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET > {

    

    
      
        
          ::rust::std::string::String greet(
            
          ) const ;
        
      
    

  }; // struct FieldRefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >


} // namespace rust


  
  
  

  namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr< ::rust::Unit >(const ::rust::Unit& t) ;
    template<>
    inline void __zngur_internal_check_init< ::rust::Unit >(const ::rust::Unit& t) ;
    template<>
    inline void __zngur_internal_assume_init< ::rust::Unit >(::rust::Unit& t) ;
    template<>
    inline void __zngur_internal_assume_deinit< ::rust::Unit >(::rust::Unit& t) ;
    template<>
    inline size_t __zngur_internal_size_of< ::rust::Unit >() ;
  }

  namespace rust {

    
      template<> struct Tuple<> { ::std::array< ::uint8_t, 1> data; };
    

} // namespace rust

namespace rust {


  template<>
  inline size_t __zngur_internal_size_of< ::rust::Unit >() {
      return 0;
  }



  
    template<>
    inline void __zngur_internal_check_init< ::rust::Unit >(const ::rust::Unit&) {}

    template<>
    inline void __zngur_internal_assume_init< ::rust::Unit >(::rust::Unit&) {}

    template<>
    inline void __zngur_internal_assume_deinit< ::rust::Unit >(::rust::Unit&) {}
  

  template<>
  inline uint8_t* __zngur_internal_data_ptr< ::rust::Unit >(::rust::Unit const & t) {
      return const_cast<uint8_t*>(&t.data[0]);
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::Unit > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::RefMut< ::rust::Unit > >(const ::rust::RefMut< ::rust::Unit >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::Unit > >;
  public:
    RefMut() {
      data =  0 ;
    }

    friend Ref< ::rust::Unit >;

    
      
    

    
      RefMut(const ::rust::Unit& t) {
        ::rust::__zngur_internal_check_init< ::rust::Unit >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }
    

    
      template<size_t OFFSET>
      RefMut(const FieldOwned< ::rust::Unit, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }
      template<size_t OFFSET>
      RefMut(const FieldRefMut< ::rust::Unit, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
      

    

    

    

  }; // struct RefMut< ::rust::Unit >

  template<>
  inline uint8_t* __zngur_internal_data_ptr< RefMut < ::rust::Unit > >(const RefMut< ::rust::Unit >& t) {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }

  template<>
  inline void __zngur_internal_assume_init< RefMut < ::rust::Unit > >(RefMut< ::rust::Unit >&) {}

  template<>
  inline void __zngur_internal_check_init< RefMut < ::rust::Unit > >(const RefMut< ::rust::Unit >&) {}

  template<>
  inline void __zngur_internal_assume_deinit< RefMut < ::rust::Unit > >(RefMut< ::rust::Unit >&) {}

  template<>
  inline size_t __zngur_internal_size_of< RefMut < ::rust::Unit > >() {
      return 8;
  }

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::Unit > {
  private:
     size_t  data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Ref< ::rust::Unit > >(const ::rust::Ref< ::rust::Unit >& t) ;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::Unit > >;
  public:
    Ref() {
      data =  0 ;
    }

    
      Ref(const ::rust::Unit& t) {
        ::rust::__zngur_internal_check_init< ::rust::Unit >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }

      

    

      Ref(RefMut< ::rust::Unit > rm) {
          data = rm.data;
      }

    
      template<size_t OFFSET>
      Ref(const FieldOwned< ::rust::Unit, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRef< ::rust::Unit, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRefMut< ::rust::Unit, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
    

    

    

    

    
};



template<>
inline uint8_t* __zngur_internal_data_ptr< Ref < ::rust::Unit > >(const Ref < ::rust::Unit >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Ref < ::rust::Unit > >(Ref < ::rust::Unit >&) {}

template<>
inline void __zngur_internal_check_init< Ref < ::rust::Unit > >(const Ref < ::rust::Unit >&) {}

template<>
inline void __zngur_internal_assume_deinit< Ref < ::rust::Unit > >(Ref < ::rust::Unit >&) {}

template<>
inline size_t __zngur_internal_size_of< Ref < ::rust::Unit > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< Raw < ::rust::Unit > >(const Raw < ::rust::Unit >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< Raw < ::rust::Unit > >(Raw < ::rust::Unit >&) {}

template<>
inline void __zngur_internal_check_init< Raw < ::rust::Unit > >(const Raw < ::rust::Unit >&) {}

template<>
inline void __zngur_internal_assume_deinit< Raw < ::rust::Unit > >(Raw < ::rust::Unit >&) {}

template<>
inline size_t __zngur_internal_size_of< Raw < ::rust::Unit > >() {
    return 8;
}



template<>
inline uint8_t* __zngur_internal_data_ptr< RawMut < ::rust::Unit > >(const RawMut < ::rust::Unit >& t) {
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}

template<>
inline void __zngur_internal_assume_init< RawMut < ::rust::Unit > >(RawMut < ::rust::Unit >&) {}

template<>
inline void __zngur_internal_check_init< RawMut < ::rust::Unit > >(const RawMut < ::rust::Unit >&) {}

template<>
inline void __zngur_internal_assume_deinit< RawMut < ::rust::Unit > >(RawMut < ::rust::Unit >&) {}

template<>
inline size_t __zngur_internal_size_of< RawMut < ::rust::Unit > >() {
    return 8;
}



} // namespace rust



namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::Unit, OFFSET > {

    

    

  }; // struct FieldOwned< ::rust::Unit, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::Unit, OFFSET > {

    

    

  }; // struct FieldRef< ::rust::Unit, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::Unit, OFFSET > {

    

    

  }; // struct FieldRefMut< ::rust::Unit, OFFSET >


} // namespace rust





  
  

  

  
  
  

  
  
  

  
  
    
    
    inline ::rust::std::string::String rust::Str::to_owned (
      ::rust::Ref< ::rust::Str > i0
    ) {
      ::rust::std::string::String o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur__str_to_owned___x7n11m20y21 (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
            if (_zngur__detect_panic_z7()) {
                _zngur__take_panic_z7();
                throw ::rust::Panic{};
            }
            
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }

    
      
      

      
        
        template<size_t OFFSET>
        inline ::rust::std::string::String rust::FieldOwned< ::rust::Str, OFFSET >::to_owned(
            
        ) const {
          return rust::Str::to_owned(
            *this
            
          );
        }
      
        
        template<size_t OFFSET>
        inline ::rust::std::string::String rust::FieldRefMut< ::rust::Str, OFFSET >::to_owned(
            
        ) const {
          return rust::Str::to_owned(
            *this
            
          );
        }
      
        
        template<size_t OFFSET>
        inline ::rust::std::string::String rust::FieldRef< ::rust::Str, OFFSET >::to_owned(
            
        ) const {
          return rust::Str::to_owned(
            *this
            
          );
        }
      

      
        
        inline ::rust::std::string::String rust::Ref< ::rust::Str >::to_owned(
            
        ) const {
          return rust::Str::to_owned(
            *this
            
          );
        }
      
        
        inline ::rust::std::string::String rust::RefMut< ::rust::Str >::to_owned(
            
        ) const {
          return rust::Str::to_owned(
            *this
            
          );
        }
      

    

    
  

namespace rust {

  
    
  
} // namespace rust



  
  

  

  
  
  

  
  
  

  
  

namespace rust {

  
    
      
        template<>
        struct ZngurPrettyPrinter< ::rust::std::string::String > {
          static inline void print( ::rust::std::string::String const& t) {
            ::rust::__zngur_internal_check_init< ::rust::std::string::String >(t);
            _zngur__std_string_String_debug_pretty_s7s11s18e25(&t.data[0]);
          }
        };

        template<>
        struct ZngurPrettyPrinter< Ref< ::rust::std::string::String > > {
          static inline void print(Ref< ::rust::std::string::String > const& t) {
            ::rust::__zngur_internal_check_init< Ref< ::rust::std::string::String > >(t);
            _zngur__std_string_String_debug_pretty_s7s11s18e25(reinterpret_cast<uint8_t*>(t.data));
          }
        };

        template<>
        struct ZngurPrettyPrinter< RefMut< ::rust::std::string::String > > {
          static inline void print(RefMut< ::rust::std::string::String > const& t) {
            ::rust::__zngur_internal_check_init< RefMut< ::rust::std::string::String > >(t);
            _zngur__std_string_String_debug_pretty_s7s11s18e25(reinterpret_cast<uint8_t*>(t.data));
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldOwned< ::rust::std::string::String, OFFSET > > {
          static inline void print(FieldOwned< ::rust::std::string::String, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::std::string::String > >::print(t);
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldRef< ::rust::std::string::String, OFFSET > > {
          static inline void print(FieldRef< ::rust::std::string::String, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::std::string::String > >::print(t);
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldRefMut< ::rust::std::string::String, OFFSET > > {
          static inline void print(FieldRefMut< ::rust::std::string::String, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::std::string::String > >::print(t);
          }
        };
      
    
  
    
  
} // namespace rust



  
  

  
    
    inline rust::crate::Person::Person(::rust::std::string::String i0) {
      ::rust::__zngur_internal_assume_init(*this);
      _zngur_crate_Person_s12(
        
          ::rust::__zngur_internal_data_ptr(i0),
        
        ::rust::__zngur_internal_data_ptr(*this)
      );
      
        ::rust::__zngur_internal_assume_deinit(i0);
      
    }
  

  
  
  

  
  
  

  
  

namespace rust {

  
    
      
        template<>
        struct ZngurPrettyPrinter< ::rust::crate::Person > {
          static inline void print( ::rust::crate::Person const& t) {
            ::rust::__zngur_internal_check_init< ::rust::crate::Person >(t);
            _zngur_crate_Person_debug_pretty_s12e19(&t.data[0]);
          }
        };

        template<>
        struct ZngurPrettyPrinter< Ref< ::rust::crate::Person > > {
          static inline void print(Ref< ::rust::crate::Person > const& t) {
            ::rust::__zngur_internal_check_init< Ref< ::rust::crate::Person > >(t);
            _zngur_crate_Person_debug_pretty_s12e19(reinterpret_cast<uint8_t*>(t.data));
          }
        };

        template<>
        struct ZngurPrettyPrinter< RefMut< ::rust::crate::Person > > {
          static inline void print(RefMut< ::rust::crate::Person > const& t) {
            ::rust::__zngur_internal_check_init< RefMut< ::rust::crate::Person > >(t);
            _zngur_crate_Person_debug_pretty_s12e19(reinterpret_cast<uint8_t*>(t.data));
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldOwned< ::rust::crate::Person, OFFSET > > {
          static inline void print(FieldOwned< ::rust::crate::Person, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::crate::Person > >::print(t);
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldRef< ::rust::crate::Person, OFFSET > > {
          static inline void print(FieldRef< ::rust::crate::Person, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::crate::Person > >::print(t);
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldRefMut< ::rust::crate::Person, OFFSET > > {
          static inline void print(FieldRefMut< ::rust::crate::Person, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::crate::Person > >::print(t);
          }
        };
      
    
  
    
  
} // namespace rust



  
  

  
    
    inline rust::crate::Robot::Robot(::uint32_t i0) {
      ::rust::__zngur_internal_assume_init(*this);
      _zngur_crate_Robot_s12(
        
          ::rust::__zngur_internal_data_ptr(i0),
        
        ::rust::__zngur_internal_data_ptr(*this)
      );
      
        ::rust::__zngur_internal_assume_deinit(i0);
      
    }
  

  
  
  

  
  
  

  
  

namespace rust {

  
    
      
        template<>
        struct ZngurPrettyPrinter< ::rust::crate::Robot > {
          static inline void print( ::rust::crate::Robot const& t) {
            ::rust::__zngur_internal_check_init< ::rust::crate::Robot >(t);
            _zngur_crate_Robot_debug_pretty_s12e18(&t.data[0]);
          }
        };

        template<>
        struct ZngurPrettyPrinter< Ref< ::rust::crate::Robot > > {
          static inline void print(Ref< ::rust::crate::Robot > const& t) {
            ::rust::__zngur_internal_check_init< Ref< ::rust::crate::Robot > >(t);
            _zngur_crate_Robot_debug_pretty_s12e18(reinterpret_cast<uint8_t*>(t.data));
          }
        };

        template<>
        struct ZngurPrettyPrinter< RefMut< ::rust::crate::Robot > > {
          static inline void print(RefMut< ::rust::crate::Robot > const& t) {
            ::rust::__zngur_internal_check_init< RefMut< ::rust::crate::Robot > >(t);
            _zngur_crate_Robot_debug_pretty_s12e18(reinterpret_cast<uint8_t*>(t.data));
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldOwned< ::rust::crate::Robot, OFFSET > > {
          static inline void print(FieldOwned< ::rust::crate::Robot, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::crate::Robot > >::print(t);
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldRef< ::rust::crate::Robot, OFFSET > > {
          static inline void print(FieldRef< ::rust::crate::Robot, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::crate::Robot > >::print(t);
          }
        };

        template<size_t OFFSET>
        struct ZngurPrettyPrinter< FieldRefMut< ::rust::crate::Robot, OFFSET > > {
          static inline void print(FieldRefMut< ::rust::crate::Robot, OFFSET > const& t) {
            ZngurPrettyPrinter< Ref< ::rust::crate::Robot > >::print(t);
          }
        };
      
    
  
    
  
} // namespace rust



  
  

  

  
  
    template <typename T, typename... Args>
    rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::make_box(Args&&... args) {
      auto data = new T(::std::forward<Args>(args)...);
      auto data_as_impl = dynamic_cast< ::rust::crate::Greeter*>(data);
      rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > o;
      ::rust::__zngur_internal_assume_init(o);
      _zngur_crate_Greeter_s12 (
        reinterpret_cast<uint8_t*>(data_as_impl),
        [](uint8_t *d) { delete reinterpret_cast< ::rust::crate::Greeter*>(d); },
        ::rust::__zngur_internal_data_ptr(o)
      );
      return o;
    }

  

  
  
  

  
  
    
    
    inline ::rust::std::string::String rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::greet (
      ::rust::Ref< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > > i0
    ) {
      ::rust::std::string::String o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur__Box_dyncrate_Greeter__greet___x7x11s20y28n29m35y36 (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
            if (_zngur__detect_panic_z7()) {
                _zngur__take_panic_z7();
                throw ::rust::Panic{};
            }
            
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }

    
      
      

      
        
        template<size_t OFFSET>
        inline ::rust::std::string::String rust::FieldOwned< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >::greet(
            
        ) const {
          return rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::greet(
            *this
            
          );
        }
      
        
        template<size_t OFFSET>
        inline ::rust::std::string::String rust::FieldRefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >::greet(
            
        ) const {
          return rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::greet(
            *this
            
          );
        }
      
        
        template<size_t OFFSET>
        inline ::rust::std::string::String rust::FieldRef< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >, OFFSET >::greet(
            
        ) const {
          return rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::greet(
            *this
            
          );
        }
      

      
        
        inline ::rust::std::string::String rust::Ref< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >::greet(
            
        ) const {
          return rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::greet(
            *this
            
          );
        }
      
        
        inline ::rust::std::string::String rust::RefMut< ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > >::greet(
            
        ) const {
          return rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::greet(
            *this
            
          );
        }
      

    

    
      
      

      inline ::rust::std::string::String rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::greet(
            
      )  const  {
        return rust::Box< ::rust::Dyn< ::rust::crate::Greeter > >::greet(
          *this
          
        );
      }
    
  

namespace rust {

  
    
  
} // namespace rust



  
  

  

  
  
  

  
  
  

  
  

namespace rust {

  
    
  
} // namespace rust




  namespace rust {
    namespace crate {
    
    inline ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > create_greeter_by_type(
      ::rust::Bool i0, ::rust::std::string::String i1, ::uint32_t i2
    ) {
      ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > o{};
       ::rust::__zngur_internal_assume_deinit(i0);  ::rust::__zngur_internal_assume_deinit(i1);  ::rust::__zngur_internal_assume_deinit(i2); 
      _zngur_crate_create_greeter_by_type_s12 (
        ::rust::__zngur_internal_data_ptr(i0), ::rust::__zngur_internal_data_ptr(i1), ::rust::__zngur_internal_data_ptr(i2),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
            if (_zngur__detect_panic_z7()) {
                _zngur__take_panic_z7();
                throw ::rust::Panic{};
            }
            
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }
  } // namespace rust
    } // namespace crate

  namespace rust {
    namespace crate {
    
    inline ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > create_person(
      ::rust::std::string::String i0
    ) {
      ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur_crate_create_person_s12 (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
            if (_zngur__detect_panic_z7()) {
                _zngur__take_panic_z7();
                throw ::rust::Panic{};
            }
            
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }
  } // namespace rust
    } // namespace crate

  namespace rust {
    namespace crate {
    
    inline ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > create_robot(
      ::uint32_t i0
    ) {
      ::rust::Box< ::rust::Dyn< ::rust::crate::Greeter > > o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur_crate_create_robot_s12 (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
            if (_zngur__detect_panic_z7()) {
                _zngur__take_panic_z7();
                throw ::rust::Panic{};
            }
            
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }
  } // namespace rust
    } // namespace crate

  namespace rust {
    namespace crate {
    
    inline ::rust::Unit print_greeting_person(
      ::rust::crate::Person i0
    ) {
      ::rust::Unit o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur_crate_print_greeting_person_s12 (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
            if (_zngur__detect_panic_z7()) {
                _zngur__take_panic_z7();
                throw ::rust::Panic{};
            }
            
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }
  } // namespace rust
    } // namespace crate

  namespace rust {
    namespace crate {
    
    inline ::rust::Unit print_greeting_robot(
      ::rust::crate::Robot i0
    ) {
      ::rust::Unit o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur_crate_print_greeting_robot_s12 (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
            if (_zngur__detect_panic_z7()) {
                _zngur__take_panic_z7();
                throw ::rust::Panic{};
            }
            
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }
  } // namespace rust
    } // namespace crate


namespace rust {
namespace exported_functions {



} // namespace exported_functions



} // namespace rust