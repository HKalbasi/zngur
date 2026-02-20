#pragma once

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <csignal>
#include <array>
#include <iostream>
#include <functional>
#include <math.h>







#define zngur_dbg(x) (::rust::zngur_dbg_impl(__FILE__, __LINE__, #x, x))

namespace rust {

  template<typename T>
  struct __zngur_internal {
    static inline uint8_t* data_ptr(const T& t) noexcept;
    static void assume_init(T& t) noexcept ;
    static void assume_deinit(T& t) noexcept ;
    static inline void check_init(const T&) noexcept;
    static inline size_t size_of() noexcept ;
  };

  template<typename T>
  inline uint8_t* __zngur_internal_data_ptr(const T& t) noexcept {
    return __zngur_internal<T>::data_ptr(t);
  }

  template<typename T>
  void __zngur_internal_assume_init(T& t) noexcept {
    __zngur_internal<T>::assume_init(t);
  }

  template<typename T>
  void __zngur_internal_assume_deinit(T& t) noexcept {
    __zngur_internal<T>::assume_deinit(t);
  }

  template<typename T>
  inline size_t __zngur_internal_size_of() noexcept {
    return __zngur_internal<T>::size_of();
  }

  template<typename T>
  inline void __zngur_internal_move_to_rust(uint8_t* dst, T& t) noexcept {
    memcpy(dst, ::rust::__zngur_internal_data_ptr(t), ::rust::__zngur_internal_size_of<T>());
    ::rust::__zngur_internal_assume_deinit(t);
  }

  template<typename T>
  inline T __zngur_internal_move_from_rust(uint8_t* src) noexcept {
    T t;
    ::rust::__zngur_internal_assume_init(t);
    memcpy(::rust::__zngur_internal_data_ptr(t), src, ::rust::__zngur_internal_size_of<T>());
    return t;
  }

  template<typename T>
  inline void __zngur_internal_check_init(const T& t) noexcept {
    __zngur_internal<T>::check_init(t);
  }

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
    inline operator T() const noexcept { return *::rust::Ref<T>(*this); }
  };

  template<typename T, size_t OFFSET>
  struct FieldRef {
    inline operator T() const noexcept { return *::rust::Ref<T>(*this); }
  };

  template<typename T, size_t OFFSET>
  struct FieldRefMut {
    inline operator T() const noexcept { return *::rust::Ref<T>(*this); }
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

  // specializations for Refs of Refs

  

  template<typename T>
  struct Ref < Ref < T > > {
    Ref() {
      data = 0;
    }
    Ref(const Ref < T >& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< Ref < T >, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    
    template<size_t OFFSET>
    Ref(const FieldRef< Ref < T >, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }
    

    template<size_t OFFSET>
    Ref(const FieldRefMut< Ref < T >, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    Ref< T >& operator*() {
      return *reinterpret_cast< Ref < T >*>(data);
    }

  private:
    size_t data;
    friend ::rust::__zngur_internal< Ref < Ref < T > > >;
    friend ::rust::ZngurPrettyPrinter< Ref < Ref < T > > >;

  };

  template<typename T>
  struct __zngur_internal< Ref < Ref < T > > > {
    static inline uint8_t* data_ptr(const Ref < Ref < T > >& t) noexcept {
        return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
    }
    static inline void assume_init(Ref < Ref < T > >&) noexcept {}

    static inline void check_init(const Ref < Ref < T > >&) noexcept {}

    static inline void assume_deinit(Ref < Ref < T > >&) noexcept {}

    static inline size_t size_of() noexcept {
        return __zngur_internal_size_of< Ref < T > >({});
    }
  };

  template<typename T>
  struct ZngurPrettyPrinter< Ref < Ref < T > > > {
    static inline void print(Ref < Ref < T > > const& t) {
      ::rust::__zngur_internal_check_init(t);
      ::rust::ZngurPrettyPrinter< Ref < T > >::print( reinterpret_cast< const Ref < T > &>(t.data) );
    }
  };

  

  template<typename T>
  struct Ref < RefMut < T > > {
    Ref() {
      data = 0;
    }
    Ref(const RefMut < T >& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< RefMut < T >, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    
    template<size_t OFFSET>
    Ref(const FieldRef< RefMut < T >, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }
    

    template<size_t OFFSET>
    Ref(const FieldRefMut< RefMut < T >, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    RefMut< T >& operator*() {
      return *reinterpret_cast< RefMut < T >*>(data);
    }

  private:
    size_t data;
    friend ::rust::__zngur_internal< Ref < RefMut < T > > >;
    friend ::rust::ZngurPrettyPrinter< Ref < RefMut < T > > >;

  };

  template<typename T>
  struct __zngur_internal< Ref < RefMut < T > > > {
    static inline uint8_t* data_ptr(const Ref < RefMut < T > >& t) noexcept {
        return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
    }
    static inline void assume_init(Ref < RefMut < T > >&) noexcept {}

    static inline void check_init(const Ref < RefMut < T > >&) noexcept {}

    static inline void assume_deinit(Ref < RefMut < T > >&) noexcept {}

    static inline size_t size_of() noexcept {
        return __zngur_internal_size_of< RefMut < T > >({});
    }
  };

  template<typename T>
  struct ZngurPrettyPrinter< Ref < RefMut < T > > > {
    static inline void print(Ref < RefMut < T > > const& t) {
      ::rust::__zngur_internal_check_init(t);
      ::rust::ZngurPrettyPrinter< RefMut < T > >::print( reinterpret_cast< const RefMut < T > &>(t.data) );
    }
  };

  

  

  template<typename T>
  struct RefMut < Ref < T > > {
    RefMut() {
      data = 0;
    }
    RefMut(const Ref < T >& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< Ref < T >, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    

    template<size_t OFFSET>
    RefMut(const FieldRefMut< Ref < T >, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    Ref< T >& operator*() {
      return *reinterpret_cast< Ref < T >*>(data);
    }

  private:
    size_t data;
    friend ::rust::__zngur_internal< RefMut < Ref < T > > >;
    friend ::rust::ZngurPrettyPrinter< RefMut < Ref < T > > >;

  };

  template<typename T>
  struct __zngur_internal< RefMut < Ref < T > > > {
    static inline uint8_t* data_ptr(const RefMut < Ref < T > >& t) noexcept {
        return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
    }
    static inline void assume_init(RefMut < Ref < T > >&) noexcept {}

    static inline void check_init(const RefMut < Ref < T > >&) noexcept {}

    static inline void assume_deinit(RefMut < Ref < T > >&) noexcept {}

    static inline size_t size_of() noexcept {
        return __zngur_internal_size_of< Ref < T > >({});
    }
  };

  template<typename T>
  struct ZngurPrettyPrinter< RefMut < Ref < T > > > {
    static inline void print(RefMut < Ref < T > > const& t) {
      ::rust::__zngur_internal_check_init(t);
      ::rust::ZngurPrettyPrinter< Ref < T > >::print( reinterpret_cast< const Ref < T > &>(t.data) );
    }
  };

  

  template<typename T>
  struct RefMut < RefMut < T > > {
    RefMut() {
      data = 0;
    }
    RefMut(const RefMut < T >& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< RefMut < T >, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    

    template<size_t OFFSET>
    RefMut(const FieldRefMut< RefMut < T >, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    RefMut< T >& operator*() {
      return *reinterpret_cast< RefMut < T >*>(data);
    }

  private:
    size_t data;
    friend ::rust::__zngur_internal< RefMut < RefMut < T > > >;
    friend ::rust::ZngurPrettyPrinter< RefMut < RefMut < T > > >;

  };

  template<typename T>
  struct __zngur_internal< RefMut < RefMut < T > > > {
    static inline uint8_t* data_ptr(const RefMut < RefMut < T > >& t) noexcept {
        return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
    }
    static inline void assume_init(RefMut < RefMut < T > >&) noexcept {}

    static inline void check_init(const RefMut < RefMut < T > >&) noexcept {}

    static inline void assume_deinit(RefMut < RefMut < T > >&) noexcept {}

    static inline size_t size_of() noexcept {
        return __zngur_internal_size_of< RefMut < T > >({});
    }
  };

  template<typename T>
  struct ZngurPrettyPrinter< RefMut < RefMut < T > > > {
    static inline void print(RefMut < RefMut < T > > const& t) {
      ::rust::__zngur_internal_check_init(t);
      ::rust::ZngurPrettyPrinter< RefMut < T > >::print( reinterpret_cast< const RefMut < T > &>(t.data) );
    }
  };

  



  
  

  template<>
  struct __zngur_internal< int8_t > {
    static inline uint8_t* data_ptr(const int8_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int8_t&) noexcept {}
    static inline void assume_deinit(int8_t&) noexcept {}
    static inline void check_init(int8_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int8_t);
    }
  };

  template<>
  struct __zngur_internal< int8_t* > {
    static inline uint8_t* data_ptr(int8_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int8_t*&) noexcept {}
    static inline void assume_deinit(int8_t*&) noexcept {}
    static inline void check_init(int8_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int8_t);
    }
  };

  template<>
  struct __zngur_internal< int8_t const* > {
    static inline uint8_t* data_ptr(int8_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int8_t const*&) noexcept {}
    static inline void assume_deinit(int8_t const*&) noexcept {}
    static inline void check_init(int8_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int8_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< int8_t > >;
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
    friend ::rust::__zngur_internal<RefMut< int8_t > >;
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
  struct __zngur_internal< ::rust::Ref<int8_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<int8_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int8_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int8_t>&) noexcept {}
    static inline void check_init(::rust::Ref<int8_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int8_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<int8_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<int8_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int8_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int8_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<int8_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int8_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<int8_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<int8_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int8_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int8_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<int8_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int8_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<int8_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<int8_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int8_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<int8_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<int8_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int8_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int8_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<int8_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int8_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<int8_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<int8_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int8_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int8_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<int8_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int8_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<int8_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<int8_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int8_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int8_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<int8_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int8_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<int8_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<int8_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int8_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< uint8_t > {
    static inline uint8_t* data_ptr(const uint8_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint8_t&) noexcept {}
    static inline void assume_deinit(uint8_t&) noexcept {}
    static inline void check_init(uint8_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint8_t);
    }
  };

  template<>
  struct __zngur_internal< uint8_t* > {
    static inline uint8_t* data_ptr(uint8_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint8_t*&) noexcept {}
    static inline void assume_deinit(uint8_t*&) noexcept {}
    static inline void check_init(uint8_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint8_t);
    }
  };

  template<>
  struct __zngur_internal< uint8_t const* > {
    static inline uint8_t* data_ptr(uint8_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint8_t const*&) noexcept {}
    static inline void assume_deinit(uint8_t const*&) noexcept {}
    static inline void check_init(uint8_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint8_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< uint8_t > >;
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
    friend ::rust::__zngur_internal<RefMut< uint8_t > >;
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
  struct __zngur_internal< ::rust::Ref<uint8_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<uint8_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint8_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint8_t>&) noexcept {}
    static inline void check_init(::rust::Ref<uint8_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint8_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<uint8_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<uint8_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint8_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint8_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<uint8_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint8_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<uint8_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<uint8_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint8_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint8_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<uint8_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint8_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<uint8_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<uint8_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint8_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<uint8_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<uint8_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint8_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint8_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<uint8_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint8_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<uint8_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<uint8_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint8_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint8_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<uint8_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint8_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<uint8_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<uint8_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint8_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint8_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<uint8_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint8_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<uint8_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<uint8_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint8_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< int16_t > {
    static inline uint8_t* data_ptr(const int16_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int16_t&) noexcept {}
    static inline void assume_deinit(int16_t&) noexcept {}
    static inline void check_init(int16_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int16_t);
    }
  };

  template<>
  struct __zngur_internal< int16_t* > {
    static inline uint8_t* data_ptr(int16_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int16_t*&) noexcept {}
    static inline void assume_deinit(int16_t*&) noexcept {}
    static inline void check_init(int16_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int16_t);
    }
  };

  template<>
  struct __zngur_internal< int16_t const* > {
    static inline uint8_t* data_ptr(int16_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int16_t const*&) noexcept {}
    static inline void assume_deinit(int16_t const*&) noexcept {}
    static inline void check_init(int16_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int16_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< int16_t > >;
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
    friend ::rust::__zngur_internal<RefMut< int16_t > >;
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
  struct __zngur_internal< ::rust::Ref<int16_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<int16_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int16_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int16_t>&) noexcept {}
    static inline void check_init(::rust::Ref<int16_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int16_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<int16_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<int16_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int16_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int16_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<int16_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int16_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<int16_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<int16_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int16_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int16_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<int16_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int16_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<int16_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<int16_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int16_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<int16_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<int16_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int16_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int16_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<int16_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int16_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<int16_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<int16_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int16_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int16_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<int16_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int16_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<int16_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<int16_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int16_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int16_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<int16_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int16_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<int16_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<int16_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int16_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< uint16_t > {
    static inline uint8_t* data_ptr(const uint16_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint16_t&) noexcept {}
    static inline void assume_deinit(uint16_t&) noexcept {}
    static inline void check_init(uint16_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint16_t);
    }
  };

  template<>
  struct __zngur_internal< uint16_t* > {
    static inline uint8_t* data_ptr(uint16_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint16_t*&) noexcept {}
    static inline void assume_deinit(uint16_t*&) noexcept {}
    static inline void check_init(uint16_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint16_t);
    }
  };

  template<>
  struct __zngur_internal< uint16_t const* > {
    static inline uint8_t* data_ptr(uint16_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint16_t const*&) noexcept {}
    static inline void assume_deinit(uint16_t const*&) noexcept {}
    static inline void check_init(uint16_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint16_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< uint16_t > >;
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
    friend ::rust::__zngur_internal<RefMut< uint16_t > >;
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
  struct __zngur_internal< ::rust::Ref<uint16_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<uint16_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint16_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint16_t>&) noexcept {}
    static inline void check_init(::rust::Ref<uint16_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint16_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<uint16_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<uint16_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint16_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint16_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<uint16_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint16_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<uint16_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<uint16_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint16_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint16_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<uint16_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint16_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<uint16_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<uint16_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint16_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<uint16_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<uint16_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint16_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint16_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<uint16_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint16_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<uint16_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<uint16_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint16_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint16_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<uint16_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint16_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<uint16_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<uint16_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint16_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint16_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<uint16_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint16_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<uint16_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<uint16_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint16_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< int32_t > {
    static inline uint8_t* data_ptr(const int32_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int32_t&) noexcept {}
    static inline void assume_deinit(int32_t&) noexcept {}
    static inline void check_init(int32_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int32_t);
    }
  };

  template<>
  struct __zngur_internal< int32_t* > {
    static inline uint8_t* data_ptr(int32_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int32_t*&) noexcept {}
    static inline void assume_deinit(int32_t*&) noexcept {}
    static inline void check_init(int32_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int32_t);
    }
  };

  template<>
  struct __zngur_internal< int32_t const* > {
    static inline uint8_t* data_ptr(int32_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int32_t const*&) noexcept {}
    static inline void assume_deinit(int32_t const*&) noexcept {}
    static inline void check_init(int32_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int32_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< int32_t > >;
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
    friend ::rust::__zngur_internal<RefMut< int32_t > >;
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
  struct __zngur_internal< ::rust::Ref<int32_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<int32_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int32_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int32_t>&) noexcept {}
    static inline void check_init(::rust::Ref<int32_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int32_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<int32_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<int32_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int32_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int32_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<int32_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int32_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<int32_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<int32_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int32_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int32_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<int32_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int32_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<int32_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<int32_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int32_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<int32_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<int32_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int32_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int32_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<int32_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int32_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<int32_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<int32_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int32_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int32_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<int32_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int32_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<int32_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<int32_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int32_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int32_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<int32_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int32_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<int32_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<int32_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int32_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< uint32_t > {
    static inline uint8_t* data_ptr(const uint32_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint32_t&) noexcept {}
    static inline void assume_deinit(uint32_t&) noexcept {}
    static inline void check_init(uint32_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint32_t);
    }
  };

  template<>
  struct __zngur_internal< uint32_t* > {
    static inline uint8_t* data_ptr(uint32_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint32_t*&) noexcept {}
    static inline void assume_deinit(uint32_t*&) noexcept {}
    static inline void check_init(uint32_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint32_t);
    }
  };

  template<>
  struct __zngur_internal< uint32_t const* > {
    static inline uint8_t* data_ptr(uint32_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint32_t const*&) noexcept {}
    static inline void assume_deinit(uint32_t const*&) noexcept {}
    static inline void check_init(uint32_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint32_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< uint32_t > >;
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
    friend ::rust::__zngur_internal<RefMut< uint32_t > >;
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
  struct __zngur_internal< ::rust::Ref<uint32_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<uint32_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint32_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint32_t>&) noexcept {}
    static inline void check_init(::rust::Ref<uint32_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint32_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<uint32_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<uint32_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint32_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint32_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<uint32_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint32_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<uint32_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<uint32_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint32_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint32_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<uint32_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint32_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<uint32_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<uint32_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint32_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<uint32_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<uint32_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint32_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint32_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<uint32_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint32_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<uint32_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<uint32_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint32_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint32_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<uint32_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint32_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<uint32_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<uint32_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint32_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint32_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<uint32_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint32_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<uint32_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<uint32_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint32_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< int64_t > {
    static inline uint8_t* data_ptr(const int64_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int64_t&) noexcept {}
    static inline void assume_deinit(int64_t&) noexcept {}
    static inline void check_init(int64_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int64_t);
    }
  };

  template<>
  struct __zngur_internal< int64_t* > {
    static inline uint8_t* data_ptr(int64_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int64_t*&) noexcept {}
    static inline void assume_deinit(int64_t*&) noexcept {}
    static inline void check_init(int64_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int64_t);
    }
  };

  template<>
  struct __zngur_internal< int64_t const* > {
    static inline uint8_t* data_ptr(int64_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(int64_t const*&) noexcept {}
    static inline void assume_deinit(int64_t const*&) noexcept {}
    static inline void check_init(int64_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(int64_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< int64_t > >;
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
    friend ::rust::__zngur_internal<RefMut< int64_t > >;
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
  struct __zngur_internal< ::rust::Ref<int64_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<int64_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int64_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int64_t>&) noexcept {}
    static inline void check_init(::rust::Ref<int64_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int64_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<int64_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<int64_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int64_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int64_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<int64_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int64_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<int64_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<int64_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<int64_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<int64_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<int64_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<int64_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<int64_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<int64_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<int64_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<int64_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<int64_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int64_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int64_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<int64_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int64_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<int64_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<int64_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int64_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int64_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<int64_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int64_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<int64_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<int64_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<int64_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<int64_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<int64_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<int64_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<int64_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<int64_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<int64_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< uint64_t > {
    static inline uint8_t* data_ptr(const uint64_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint64_t&) noexcept {}
    static inline void assume_deinit(uint64_t&) noexcept {}
    static inline void check_init(uint64_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint64_t);
    }
  };

  template<>
  struct __zngur_internal< uint64_t* > {
    static inline uint8_t* data_ptr(uint64_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint64_t*&) noexcept {}
    static inline void assume_deinit(uint64_t*&) noexcept {}
    static inline void check_init(uint64_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint64_t);
    }
  };

  template<>
  struct __zngur_internal< uint64_t const* > {
    static inline uint8_t* data_ptr(uint64_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(uint64_t const*&) noexcept {}
    static inline void assume_deinit(uint64_t const*&) noexcept {}
    static inline void check_init(uint64_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(uint64_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< uint64_t > >;
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
    friend ::rust::__zngur_internal<RefMut< uint64_t > >;
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
  struct __zngur_internal< ::rust::Ref<uint64_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<uint64_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint64_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint64_t>&) noexcept {}
    static inline void check_init(::rust::Ref<uint64_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint64_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<uint64_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<uint64_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint64_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint64_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<uint64_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint64_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<uint64_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<uint64_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<uint64_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<uint64_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<uint64_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<uint64_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<uint64_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<uint64_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<uint64_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<uint64_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<uint64_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint64_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint64_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<uint64_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint64_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<uint64_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<uint64_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint64_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint64_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<uint64_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint64_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<uint64_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<uint64_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<uint64_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<uint64_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<uint64_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<uint64_t>);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<uint64_t> > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<uint64_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<uint64_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::double_t > {
    static inline uint8_t* data_ptr(const ::double_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::double_t&) noexcept {}
    static inline void assume_deinit(::double_t&) noexcept {}
    static inline void check_init(::double_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::double_t);
    }
  };

  template<>
  struct __zngur_internal< ::double_t* > {
    static inline uint8_t* data_ptr(::double_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::double_t*&) noexcept {}
    static inline void assume_deinit(::double_t*&) noexcept {}
    static inline void check_init(::double_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::double_t);
    }
  };

  template<>
  struct __zngur_internal< ::double_t const* > {
    static inline uint8_t* data_ptr(::double_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::double_t const*&) noexcept {}
    static inline void assume_deinit(::double_t const*&) noexcept {}
    static inline void check_init(::double_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::double_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::double_t > >;
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
    friend ::rust::__zngur_internal<RefMut< ::double_t > >;
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
  struct __zngur_internal< ::rust::Ref<::double_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<::double_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<::double_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<::double_t>&) noexcept {}
    static inline void check_init(::rust::Ref<::double_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<::double_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<::double_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<::double_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<::double_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<::double_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<::double_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<::double_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<::double_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<::double_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<::double_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<::double_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<::double_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<::double_t>);
    }
  };


  template<>
  struct Ref< ::rust::Ref<::double_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<::double_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<::double_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<::double_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<::double_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<::double_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<::double_t>*>(data);
    }

  private:
    size_t data;
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<::double_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<::double_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<::double_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<::double_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<::double_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<::double_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<::double_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<::double_t>*>(data);
    }
  private:
    size_t data;
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<::double_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<::double_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<::double_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<::double_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<::double_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<::double_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<::double_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<::double_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<::double_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<::double_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<::double_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<::double_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<::double_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<::double_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<::double_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<::double_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<::double_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<::double_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<::double_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<::double_t>);
    }
  };


  template<>
  struct Ref< ::rust::RefMut<::double_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<::double_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<::double_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<::double_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<::double_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<::double_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<::double_t>*>(data);
    }

  private:
    size_t data;
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<::double_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<::double_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<::double_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<::double_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<::double_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<::double_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<::double_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<::double_t>*>(data);
    }
  private:
    size_t data;
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<::double_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<::double_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::float_t > {
    static inline uint8_t* data_ptr(const ::float_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::float_t&) noexcept {}
    static inline void assume_deinit(::float_t&) noexcept {}
    static inline void check_init(::float_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::float_t);
    }
  };

  template<>
  struct __zngur_internal< ::float_t* > {
    static inline uint8_t* data_ptr(::float_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::float_t*&) noexcept {}
    static inline void assume_deinit(::float_t*&) noexcept {}
    static inline void check_init(::float_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::float_t);
    }
  };

  template<>
  struct __zngur_internal< ::float_t const* > {
    static inline uint8_t* data_ptr(::float_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::float_t const*&) noexcept {}
    static inline void assume_deinit(::float_t const*&) noexcept {}
    static inline void check_init(::float_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::float_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::float_t > >;
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
    friend ::rust::__zngur_internal<RefMut< ::float_t > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::float_t > >;
  };

  
  
    template<>
    struct ZngurPrettyPrinter< ::float_t > {
      static inline void print(::float_t const& t) {
        ::std::cerr << t << ::std::endl;
      }
    };
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::Ref<::float_t> > {
    static inline uint8_t* data_ptr(const ::rust::Ref<::float_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<::float_t>&) noexcept {}
    static inline void assume_deinit(::rust::Ref<::float_t>&) noexcept {}
    static inline void check_init(::rust::Ref<::float_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<::float_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<::float_t>* > {
    static inline uint8_t* data_ptr(::rust::Ref<::float_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<::float_t>*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<::float_t>*&) noexcept {}
    static inline void check_init(::rust::Ref<::float_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<::float_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::Ref<::float_t> const* > {
    static inline uint8_t* data_ptr(::rust::Ref<::float_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::Ref<::float_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::Ref<::float_t> const*&) noexcept {}
    static inline void check_init(::rust::Ref<::float_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::Ref<::float_t>);
    }
  };


  template<>
  struct Ref< ::rust::Ref<::float_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::Ref<::float_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::Ref<::float_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::Ref<::float_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::Ref<::float_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<::float_t>& operator*() {
      return *reinterpret_cast< ::rust::Ref<::float_t>*>(data);
    }

  private:
    size_t data;
    friend ::rust::__zngur_internal<Ref< ::rust::Ref<::float_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<::float_t> > >;

  };

  template<>
  struct RefMut< ::rust::Ref<::float_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::Ref<::float_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::Ref<::float_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::Ref<::float_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::Ref<::float_t>& operator*() {
        return *reinterpret_cast< ::rust::Ref<::float_t>*>(data);
    }
  private:
    size_t data;
    friend ::rust::__zngur_internal<RefMut< ::rust::Ref<::float_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::Ref<::float_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::RefMut<::float_t> > {
    static inline uint8_t* data_ptr(const ::rust::RefMut<::float_t>& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<::float_t>&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<::float_t>&) noexcept {}
    static inline void check_init(::rust::RefMut<::float_t>&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<::float_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<::float_t>* > {
    static inline uint8_t* data_ptr(::rust::RefMut<::float_t>* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<::float_t>*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<::float_t>*&) noexcept {}
    static inline void check_init(::rust::RefMut<::float_t>*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<::float_t>);
    }
  };

  template<>
  struct __zngur_internal< ::rust::RefMut<::float_t> const* > {
    static inline uint8_t* data_ptr(::rust::RefMut<::float_t> const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::RefMut<::float_t> const*&) noexcept {}
    static inline void assume_deinit(::rust::RefMut<::float_t> const*&) noexcept {}
    static inline void check_init(::rust::RefMut<::float_t> const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::RefMut<::float_t>);
    }
  };


  template<>
  struct Ref< ::rust::RefMut<::float_t> > {
    Ref() {
      data = 0;
    }
    Ref(const ::rust::RefMut<::float_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    Ref(const FieldOwned< ::rust::RefMut<::float_t>, OFFSET >& f) {
      data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRef< ::rust::RefMut<::float_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    Ref(const FieldRefMut< ::rust::RefMut<::float_t>, OFFSET >& f) {
      data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<::float_t>& operator*() {
      return *reinterpret_cast< ::rust::RefMut<::float_t>*>(data);
    }

  private:
    size_t data;
    friend ::rust::__zngur_internal<Ref< ::rust::RefMut<::float_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<::float_t> > >;

  };

  template<>
  struct RefMut< ::rust::RefMut<::float_t> > {
    RefMut() {
      data = 0;
    }

    RefMut(::rust::RefMut<::float_t>& t) {
      data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }

    template<size_t OFFSET>
    RefMut(const FieldOwned< ::rust::RefMut<::float_t>, OFFSET >& f) {
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }

    template<size_t OFFSET>
    RefMut(const FieldRefMut< ::rust::RefMut<::float_t>, OFFSET >& f) {
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }

    ::rust::RefMut<::float_t>& operator*() {
        return *reinterpret_cast< ::rust::RefMut<::float_t>*>(data);
    }
  private:
    size_t data;
    friend ::rust::__zngur_internal<RefMut< ::rust::RefMut<::float_t> > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::RefMut<::float_t> > >;
  };

  
  

  

// end builtin types

  
  

  template<>
  struct __zngur_internal< ::rust::ZngurCppOpaqueOwnedObject > {
    static inline uint8_t* data_ptr(const ::rust::ZngurCppOpaqueOwnedObject& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::ZngurCppOpaqueOwnedObject&) noexcept {}
    static inline void assume_deinit(::rust::ZngurCppOpaqueOwnedObject&) noexcept {}
    static inline void check_init(::rust::ZngurCppOpaqueOwnedObject&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::ZngurCppOpaqueOwnedObject);
    }
  };

  template<>
  struct __zngur_internal< ::rust::ZngurCppOpaqueOwnedObject* > {
    static inline uint8_t* data_ptr(::rust::ZngurCppOpaqueOwnedObject* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::ZngurCppOpaqueOwnedObject*&) noexcept {}
    static inline void assume_deinit(::rust::ZngurCppOpaqueOwnedObject*&) noexcept {}
    static inline void check_init(::rust::ZngurCppOpaqueOwnedObject*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::ZngurCppOpaqueOwnedObject);
    }
  };

  template<>
  struct __zngur_internal< ::rust::ZngurCppOpaqueOwnedObject const* > {
    static inline uint8_t* data_ptr(::rust::ZngurCppOpaqueOwnedObject const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::rust::ZngurCppOpaqueOwnedObject const*&) noexcept {}
    static inline void assume_deinit(::rust::ZngurCppOpaqueOwnedObject const*&) noexcept {}
    static inline void check_init(::rust::ZngurCppOpaqueOwnedObject const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::rust::ZngurCppOpaqueOwnedObject);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::rust::ZngurCppOpaqueOwnedObject > >;
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
    friend ::rust::__zngur_internal<RefMut< ::rust::ZngurCppOpaqueOwnedObject > >;
    friend ::rust::ZngurPrettyPrinter< Ref< ::rust::ZngurCppOpaqueOwnedObject > >;
  };

  
  

  

// end builtin types

  
  
    #if defined(__APPLE__) || defined(__wasm__)
  

  template<>
  struct __zngur_internal< ::size_t > {
    static inline uint8_t* data_ptr(const ::size_t& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::size_t&) noexcept {}
    static inline void assume_deinit(::size_t&) noexcept {}
    static inline void check_init(::size_t&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::size_t);
    }
  };

  template<>
  struct __zngur_internal< ::size_t* > {
    static inline uint8_t* data_ptr(::size_t* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::size_t*&) noexcept {}
    static inline void assume_deinit(::size_t*&) noexcept {}
    static inline void check_init(::size_t*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::size_t);
    }
  };

  template<>
  struct __zngur_internal< ::size_t const* > {
    static inline uint8_t* data_ptr(::size_t const* const & t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t));
    }
    static inline void assume_init(::size_t const*&) noexcept {}
    static inline void assume_deinit(::size_t const*&) noexcept {}
    static inline void check_init(::size_t const*&) noexcept {}
    static inline size_t size_of() noexcept {
      return sizeof(::size_t);
    }
  };


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
    friend ::rust::__zngur_internal<Ref< ::size_t > >;
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
    friend ::rust::__zngur_internal<RefMut< ::size_t > >;
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
  

  
    

    

    

    

    
      
      
        
      
    

    

  // end self.type_defs
  
    

    

    

    

    
      
      
        
      
    

    

  // end self.type_defs
  
    
      void _zngur__crate_CharPrinter_print___x7s13n25m31y32_a4954a65fb (
        
          uint8_t* i0,
        
        uint8_t* o
      ) noexcept ;
    
      void _zngur__crate_CharPrinter_is_alphabetic___x7s13n25m39y40_a4954a65fb (
        
          uint8_t* i0,
        
        uint8_t* o
      ) noexcept ;
    
      void _zngur__crate_CharPrinter_to_uppercase___x7s13n25m38y39_a4954a65fb (
        
          uint8_t* i0,
        
        uint8_t* o
      ) noexcept ;
    

    

    

    

    
      
      
        void _zngur_crate_CharPrinter_drop_in_place_s12e24(uint8_t*);
      
    

    

  // end self.type_defs
  
    

    

    

    

    
      
      
        
      
    

    

  // end self.type_defs
  

} // extern "C"


  namespace rust {
struct Char;
}


  namespace rust {
struct Bool;
}


  namespace rust {
namespace crate {
struct CharPrinter;
}
}


  




namespace rust {



}




  
  
  

  namespace rust {
    template<>
    struct __zngur_internal< ::rust::Char > {
      static inline uint8_t* data_ptr(const ::rust::Char& t) noexcept ;
      static inline void check_init(const ::rust::Char& t) noexcept ;
      static inline void assume_init(::rust::Char& t) noexcept ;
      static inline void assume_deinit(::rust::Char& t) noexcept ;
      static inline size_t size_of() noexcept ;
    };
  }

  namespace rust {
    struct Char {
    
      private:
        alignas(4) mutable ::std::array< ::uint8_t, 4> data;
    

    
      friend ::rust::__zngur_internal< ::rust::Char >;
      friend ::rust::ZngurPrettyPrinter< ::rust::Char >;

    

    
      public:
        operator char32_t() const {
          return *reinterpret_cast<const char32_t*>(data.data());
        }
        Char(char32_t c) {
          *reinterpret_cast<char32_t*>(data.data()) = c;
        }
    

    

    

    public:
      
        Char() {  }
        ~Char() {  }
        Char(const Char& other) {
          
          this->data = other.data;
        }
        Char& operator=(const Char& other) {
          this->data = other.data;
          return *this;
        }
        Char(Char&& other) {
          
          this->data = other.data;
        }
        Char& operator=(Char&& other) {
          this->data = other.data;
          return *this;
        }
      

      
      

      

    

    

    

    

  }; // struct Char

} // namespace rust

namespace rust {


  inline size_t __zngur_internal< ::rust::Char >::size_of() noexcept {
      return 4;
  }



  
    inline void __zngur_internal< ::rust::Char >::check_init(const ::rust::Char&) noexcept {}
    inline void __zngur_internal< ::rust::Char >::assume_init(::rust::Char&) noexcept {}
    inline void __zngur_internal< ::rust::Char >::assume_deinit(::rust::Char&) noexcept {}
  

  inline uint8_t* __zngur_internal< ::rust::Char >::data_ptr(::rust::Char const & t) noexcept {
    
      return const_cast<uint8_t*>(t.data.data());
    
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::Char > {
  private:
     size_t  data;
    friend ::rust::__zngur_internal< ::rust::RefMut< ::rust::Char > >;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::Char > >;
  public:
    RefMut() {
      data =  0 ;
    }

    friend Ref< ::rust::Char >;

    
      
    

    
      RefMut(const ::rust::Char& t) {
        ::rust::__zngur_internal_check_init< ::rust::Char >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }
    

    
      template<size_t OFFSET>
      RefMut(const FieldOwned< ::rust::Char, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }
      template<size_t OFFSET>
      RefMut(const FieldRefMut< ::rust::Char, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
      

    

    

    

  }; // struct RefMut< ::rust::Char >

  template<>
  struct __zngur_internal< RefMut < ::rust::Char > > {
    static inline uint8_t* data_ptr(const RefMut< ::rust::Char >& t) noexcept {
        return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
    }
    static inline void assume_init(RefMut< ::rust::Char >&) noexcept {}
    static inline void check_init(const RefMut< ::rust::Char >&) noexcept {}
    static inline void assume_deinit(RefMut< ::rust::Char >&) noexcept {}
    static inline size_t size_of() noexcept {
        return 8;
    }
  };

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::Char > {
  private:
     size_t  data;
    friend ::rust::__zngur_internal< ::rust::Ref< ::rust::Char > >;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::Char > >;
  public:
    Ref() {
      data =  0 ;
    }

    
      Ref(const ::rust::Char& t) {
        ::rust::__zngur_internal_check_init< ::rust::Char >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }

      

    

      Ref(RefMut< ::rust::Char > rm) {
          data = rm.data;
      }

    
      template<size_t OFFSET>
      Ref(const FieldOwned< ::rust::Char, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRef< ::rust::Char, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRefMut< ::rust::Char, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
    

    

    

    

    
};



template<>
struct __zngur_internal< Ref < ::rust::Char > > {
  static inline uint8_t* data_ptr(const Ref < ::rust::Char >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(Ref < ::rust::Char >&) noexcept {}
  static inline void check_init(const Ref < ::rust::Char >&) noexcept {}
  static inline void assume_deinit(Ref < ::rust::Char >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



template<>
struct __zngur_internal< Raw < ::rust::Char > > {
  static inline uint8_t* data_ptr(const Raw < ::rust::Char >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(Raw < ::rust::Char >&) noexcept {}
  static inline void check_init(const Raw < ::rust::Char >&) noexcept {}
  static inline void assume_deinit(Raw < ::rust::Char >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



template<>
struct __zngur_internal< RawMut < ::rust::Char > > {
  static inline uint8_t* data_ptr(const RawMut < ::rust::Char >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(RawMut < ::rust::Char >&) noexcept {}
  static inline void check_init(const RawMut < ::rust::Char >&) noexcept {}
  static inline void assume_deinit(RawMut < ::rust::Char >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



} // namespace rust




  inline ::rust::Char operator""_rs(char32_t c) {
    if (c > 0x10FFFFu || (c >= 0xD800u && c <= 0xDFFFu)) {
      return ::rust::Char(0xFFFDu);
    }
    return ::rust::Char(c);
  }


namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::Char, OFFSET > {

    

    

  }; // struct FieldOwned< ::rust::Char, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::Char, OFFSET > {

    

    

  }; // struct FieldRef< ::rust::Char, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::Char, OFFSET > {

    

    

  }; // struct FieldRefMut< ::rust::Char, OFFSET >


} // namespace rust


  
  
  

  namespace rust {
    template<>
    struct __zngur_internal< ::rust::Bool > {
      static inline uint8_t* data_ptr(const ::rust::Bool& t) noexcept ;
      static inline void check_init(const ::rust::Bool& t) noexcept ;
      static inline void assume_init(::rust::Bool& t) noexcept ;
      static inline void assume_deinit(::rust::Bool& t) noexcept ;
      static inline size_t size_of() noexcept ;
    };
  }

  namespace rust {
    struct Bool {
    
      private:
        alignas(1) mutable ::std::array< ::uint8_t, 1> data;
    

    
      friend ::rust::__zngur_internal< ::rust::Bool >;
      friend ::rust::ZngurPrettyPrinter< ::rust::Bool >;

    
      public:
        operator bool() {
          return data[0];
        }
        Bool(bool b) {
          data[0] = b;
        }
    

    

    

    

    public:
      
        Bool() {  }
        ~Bool() {  }
        Bool(const Bool& other) {
          
          this->data = other.data;
        }
        Bool& operator=(const Bool& other) {
          this->data = other.data;
          return *this;
        }
        Bool(Bool&& other) {
          
          this->data = other.data;
        }
        Bool& operator=(Bool&& other) {
          this->data = other.data;
          return *this;
        }
      

      
      

      

    

    

    

    

  }; // struct Bool

} // namespace rust

namespace rust {


  inline size_t __zngur_internal< ::rust::Bool >::size_of() noexcept {
      return 1;
  }



  
    inline void __zngur_internal< ::rust::Bool >::check_init(const ::rust::Bool&) noexcept {}
    inline void __zngur_internal< ::rust::Bool >::assume_init(::rust::Bool&) noexcept {}
    inline void __zngur_internal< ::rust::Bool >::assume_deinit(::rust::Bool&) noexcept {}
  

  inline uint8_t* __zngur_internal< ::rust::Bool >::data_ptr(::rust::Bool const & t) noexcept {
    
      return const_cast<uint8_t*>(t.data.data());
    
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::Bool > {
  private:
     size_t  data;
    friend ::rust::__zngur_internal< ::rust::RefMut< ::rust::Bool > >;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::Bool > >;
  public:
    RefMut() {
      data =  0 ;
    }

    friend Ref< ::rust::Bool >;

    
      
    

    
      RefMut(const ::rust::Bool& t) {
        ::rust::__zngur_internal_check_init< ::rust::Bool >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }
    

    
      template<size_t OFFSET>
      RefMut(const FieldOwned< ::rust::Bool, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }
      template<size_t OFFSET>
      RefMut(const FieldRefMut< ::rust::Bool, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
      

    

    

    

  }; // struct RefMut< ::rust::Bool >

  template<>
  struct __zngur_internal< RefMut < ::rust::Bool > > {
    static inline uint8_t* data_ptr(const RefMut< ::rust::Bool >& t) noexcept {
        return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
    }
    static inline void assume_init(RefMut< ::rust::Bool >&) noexcept {}
    static inline void check_init(const RefMut< ::rust::Bool >&) noexcept {}
    static inline void assume_deinit(RefMut< ::rust::Bool >&) noexcept {}
    static inline size_t size_of() noexcept {
        return 8;
    }
  };

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::Bool > {
  private:
     size_t  data;
    friend ::rust::__zngur_internal< ::rust::Ref< ::rust::Bool > >;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::Bool > >;
  public:
    Ref() {
      data =  0 ;
    }

    
      Ref(const ::rust::Bool& t) {
        ::rust::__zngur_internal_check_init< ::rust::Bool >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }

      

    

      Ref(RefMut< ::rust::Bool > rm) {
          data = rm.data;
      }

    
      template<size_t OFFSET>
      Ref(const FieldOwned< ::rust::Bool, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRef< ::rust::Bool, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRefMut< ::rust::Bool, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
    

    

    

    

    
};



template<>
struct __zngur_internal< Ref < ::rust::Bool > > {
  static inline uint8_t* data_ptr(const Ref < ::rust::Bool >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(Ref < ::rust::Bool >&) noexcept {}
  static inline void check_init(const Ref < ::rust::Bool >&) noexcept {}
  static inline void assume_deinit(Ref < ::rust::Bool >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



template<>
struct __zngur_internal< Raw < ::rust::Bool > > {
  static inline uint8_t* data_ptr(const Raw < ::rust::Bool >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(Raw < ::rust::Bool >&) noexcept {}
  static inline void check_init(const Raw < ::rust::Bool >&) noexcept {}
  static inline void assume_deinit(Raw < ::rust::Bool >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



template<>
struct __zngur_internal< RawMut < ::rust::Bool > > {
  static inline uint8_t* data_ptr(const RawMut < ::rust::Bool >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(RawMut < ::rust::Bool >&) noexcept {}
  static inline void check_init(const RawMut < ::rust::Bool >&) noexcept {}
  static inline void assume_deinit(RawMut < ::rust::Bool >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



} // namespace rust





namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::Bool, OFFSET > {

    

    

  }; // struct FieldOwned< ::rust::Bool, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::Bool, OFFSET > {

    

    

  }; // struct FieldRef< ::rust::Bool, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::Bool, OFFSET > {

    

    

  }; // struct FieldRefMut< ::rust::Bool, OFFSET >


} // namespace rust


  
  
  

  namespace rust {
    template<>
    struct __zngur_internal< ::rust::crate::CharPrinter > {
      static inline uint8_t* data_ptr(const ::rust::crate::CharPrinter& t) noexcept ;
      static inline void check_init(const ::rust::crate::CharPrinter& t) noexcept ;
      static inline void assume_init(::rust::crate::CharPrinter& t) noexcept ;
      static inline void assume_deinit(::rust::crate::CharPrinter& t) noexcept ;
      static inline size_t size_of() noexcept ;
    };
  }

  namespace rust {
    namespace crate {
    struct CharPrinter {
    
      private:
        alignas(1) mutable ::std::array< ::uint8_t, 0> data;
    

    
      friend ::rust::__zngur_internal< ::rust::crate::CharPrinter >;
      friend ::rust::ZngurPrettyPrinter< ::rust::crate::CharPrinter >;

    

    

    
      bool drop_flag;
    

    

    public:
      
        

        CharPrinter() : drop_flag(false) {  }
        ~CharPrinter() {
          if (drop_flag) {
            _zngur_crate_CharPrinter_drop_in_place_s12e24(::rust::__zngur_internal_data_ptr(*this));
          }
          
        }
        CharPrinter(const CharPrinter& other) = delete;
        CharPrinter& operator=(const CharPrinter& other) = delete;
        CharPrinter(CharPrinter&& other) : drop_flag(false) {
          
          *this = ::std::move(other);
        }
        CharPrinter& operator=(CharPrinter&& other) {
          if (this != &other) {
            if (drop_flag) {
              _zngur_crate_CharPrinter_drop_in_place_s12e24(::rust::__zngur_internal_data_ptr(*this));
            }
            this->drop_flag = other.drop_flag;
            this->data = other.data;
            other.drop_flag = false;
          }
          return *this;
        }
      

      
      

      

    

    
        static ::rust::Unit print(
          ::rust::Char i0
        ) noexcept ;
        
    
        static ::rust::Bool is_alphabetic(
          ::rust::Char i0
        ) noexcept ;
        
    
        static ::rust::Char to_uppercase(
          ::rust::Char i0
        ) noexcept ;
        
    

    

    

  }; // struct CharPrinter

} // namespace rust
    } // namespace crate

namespace rust {


  inline size_t __zngur_internal< ::rust::crate::CharPrinter >::size_of() noexcept {
      return 0;
  }



  
    inline void __zngur_internal< ::rust::crate::CharPrinter >::check_init(const ::rust::crate::CharPrinter& t) noexcept {
        if (!t.drop_flag) {
            ::std::cerr << "Use of uninitialized or moved Zngur Rust object with type ::rust::crate::CharPrinter" << ::std::endl;
            while (true) raise(SIGSEGV);
        }
    }

    inline void __zngur_internal< ::rust::crate::CharPrinter >::assume_init(::rust::crate::CharPrinter& t) noexcept {
        t.drop_flag = true;
    }

    inline void __zngur_internal< ::rust::crate::CharPrinter >::assume_deinit(::rust::crate::CharPrinter& t) noexcept {
        ::rust::__zngur_internal_check_init< ::rust::crate::CharPrinter >(t);
        t.drop_flag = false;
    }
  

  inline uint8_t* __zngur_internal< ::rust::crate::CharPrinter >::data_ptr(::rust::crate::CharPrinter const & t) noexcept {
    
      return const_cast<uint8_t*>(t.data.data());
    
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::crate::CharPrinter > {
  private:
     size_t  data;
    friend ::rust::__zngur_internal< ::rust::RefMut< ::rust::crate::CharPrinter > >;
    friend ::rust::ZngurPrettyPrinter< ::rust::RefMut< ::rust::crate::CharPrinter > >;
  public:
    RefMut() {
      data =  0 ;
    }

    friend Ref< ::rust::crate::CharPrinter >;

    
      
    

    
      RefMut(const ::rust::crate::CharPrinter& t) {
        ::rust::__zngur_internal_check_init< ::rust::crate::CharPrinter >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }
    

    
      template<size_t OFFSET>
      RefMut(const FieldOwned< ::rust::crate::CharPrinter, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }
      template<size_t OFFSET>
      RefMut(const FieldRefMut< ::rust::crate::CharPrinter, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
      

    

    

    
      
    
      
    
      
    

  }; // struct RefMut< ::rust::crate::CharPrinter >

  template<>
  struct __zngur_internal< RefMut < ::rust::crate::CharPrinter > > {
    static inline uint8_t* data_ptr(const RefMut< ::rust::crate::CharPrinter >& t) noexcept {
        return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
    }
    static inline void assume_init(RefMut< ::rust::crate::CharPrinter >&) noexcept {}
    static inline void check_init(const RefMut< ::rust::crate::CharPrinter >&) noexcept {}
    static inline void assume_deinit(RefMut< ::rust::crate::CharPrinter >&) noexcept {}
    static inline size_t size_of() noexcept {
        return 8;
    }
  };

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::crate::CharPrinter > {
  private:
     size_t  data;
    friend ::rust::__zngur_internal< ::rust::Ref< ::rust::crate::CharPrinter > >;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< ::rust::crate::CharPrinter > >;
  public:
    Ref() {
      data =  0 ;
    }

    
      Ref(const ::rust::crate::CharPrinter& t) {
        ::rust::__zngur_internal_check_init< ::rust::crate::CharPrinter >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
      }

      

    

      Ref(RefMut< ::rust::crate::CharPrinter > rm) {
          data = rm.data;
      }

    
      template<size_t OFFSET>
      Ref(const FieldOwned< ::rust::crate::CharPrinter, OFFSET >& f) {
          data = reinterpret_cast<size_t>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRef< ::rust::crate::CharPrinter, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }

      template<size_t OFFSET>
      Ref(const FieldRefMut< ::rust::crate::CharPrinter, OFFSET >& f) {
          data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
      }
    

    
    

    

    

    
      
    
      
    
      
    

    
};



template<>
struct __zngur_internal< Ref < ::rust::crate::CharPrinter > > {
  static inline uint8_t* data_ptr(const Ref < ::rust::crate::CharPrinter >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(Ref < ::rust::crate::CharPrinter >&) noexcept {}
  static inline void check_init(const Ref < ::rust::crate::CharPrinter >&) noexcept {}
  static inline void assume_deinit(Ref < ::rust::crate::CharPrinter >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



template<>
struct __zngur_internal< Raw < ::rust::crate::CharPrinter > > {
  static inline uint8_t* data_ptr(const Raw < ::rust::crate::CharPrinter >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(Raw < ::rust::crate::CharPrinter >&) noexcept {}
  static inline void check_init(const Raw < ::rust::crate::CharPrinter >&) noexcept {}
  static inline void assume_deinit(Raw < ::rust::crate::CharPrinter >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



template<>
struct __zngur_internal< RawMut < ::rust::crate::CharPrinter > > {
  static inline uint8_t* data_ptr(const RawMut < ::rust::crate::CharPrinter >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(RawMut < ::rust::crate::CharPrinter >&) noexcept {}
  static inline void check_init(const RawMut < ::rust::crate::CharPrinter >&) noexcept {}
  static inline void assume_deinit(RawMut < ::rust::crate::CharPrinter >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



} // namespace rust





namespace rust {

// Field specializations


  template<size_t OFFSET>
  struct FieldOwned< ::rust::crate::CharPrinter, OFFSET > {

    

    
      
    
      
    
      
    

  }; // struct FieldOwned< ::rust::crate::CharPrinter, OFFSET >


  template<size_t OFFSET>
  struct FieldRef< ::rust::crate::CharPrinter, OFFSET > {

    

    
      
    
      
    
      
    

  }; // struct FieldRef< ::rust::crate::CharPrinter, OFFSET >


  template<size_t OFFSET>
  struct FieldRefMut< ::rust::crate::CharPrinter, OFFSET > {

    

    
      
    
      
    
      
    

  }; // struct FieldRefMut< ::rust::crate::CharPrinter, OFFSET >


} // namespace rust


  
  
  

  namespace rust {
    template<>
    struct __zngur_internal< ::rust::Unit > {
      static inline uint8_t* data_ptr(const ::rust::Unit& t) noexcept ;
      static inline void check_init(const ::rust::Unit& t) noexcept ;
      static inline void assume_init(::rust::Unit& t) noexcept ;
      static inline void assume_deinit(::rust::Unit& t) noexcept ;
      static inline size_t size_of() noexcept ;
    };
  }

  namespace rust {
    template<> struct Tuple<  > {
    
      private:
        alignas(1) mutable ::std::array< ::uint8_t, 0> data;
    

    
      friend ::rust::__zngur_internal< ::rust::Unit >;
      friend ::rust::ZngurPrettyPrinter< ::rust::Unit >;

    

    

    

    

    public:
      
        Tuple() {  }
        ~Tuple() {  }
        Tuple(const Tuple& other) {
          
          this->data = other.data;
        }
        Tuple& operator=(const Tuple& other) {
          this->data = other.data;
          return *this;
        }
        Tuple(Tuple&& other) {
          
          this->data = other.data;
        }
        Tuple& operator=(Tuple&& other) {
          this->data = other.data;
          return *this;
        }
      

      
      

      

    

    

    

    

  }; // template<> struct Tuple<  >

} // namespace rust

namespace rust {


  inline size_t __zngur_internal< ::rust::Unit >::size_of() noexcept {
      return 0;
  }



  
    inline void __zngur_internal< ::rust::Unit >::check_init(const ::rust::Unit&) noexcept {}
    inline void __zngur_internal< ::rust::Unit >::assume_init(::rust::Unit&) noexcept {}
    inline void __zngur_internal< ::rust::Unit >::assume_deinit(::rust::Unit&) noexcept {}
  

  inline uint8_t* __zngur_internal< ::rust::Unit >::data_ptr(::rust::Unit const & t) noexcept {
    
      return const_cast<uint8_t*>(t.data.data());
    
  }




} // namespace rust

namespace rust {

  template<>
  struct RefMut< ::rust::Unit > {
  private:
     size_t  data;
    friend ::rust::__zngur_internal< ::rust::RefMut< ::rust::Unit > >;
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
  struct __zngur_internal< RefMut < ::rust::Unit > > {
    static inline uint8_t* data_ptr(const RefMut< ::rust::Unit >& t) noexcept {
        return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
    }
    static inline void assume_init(RefMut< ::rust::Unit >&) noexcept {}
    static inline void check_init(const RefMut< ::rust::Unit >&) noexcept {}
    static inline void assume_deinit(RefMut< ::rust::Unit >&) noexcept {}
    static inline size_t size_of() noexcept {
        return 8;
    }
  };

} // namespace rust

// Ref specialization


namespace rust {

  template<>
  struct Ref< ::rust::Unit > {
  private:
     size_t  data;
    friend ::rust::__zngur_internal< ::rust::Ref< ::rust::Unit > >;
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
struct __zngur_internal< Ref < ::rust::Unit > > {
  static inline uint8_t* data_ptr(const Ref < ::rust::Unit >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(Ref < ::rust::Unit >&) noexcept {}
  static inline void check_init(const Ref < ::rust::Unit >&) noexcept {}
  static inline void assume_deinit(Ref < ::rust::Unit >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



template<>
struct __zngur_internal< Raw < ::rust::Unit > > {
  static inline uint8_t* data_ptr(const Raw < ::rust::Unit >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(Raw < ::rust::Unit >&) noexcept {}
  static inline void check_init(const Raw < ::rust::Unit >&) noexcept {}
  static inline void assume_deinit(Raw < ::rust::Unit >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



template<>
struct __zngur_internal< RawMut < ::rust::Unit > > {
  static inline uint8_t* data_ptr(const RawMut < ::rust::Unit >& t) noexcept {
      return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
  }
  static inline void assume_init(RawMut < ::rust::Unit >&) noexcept {}
  static inline void check_init(const RawMut < ::rust::Unit >&) noexcept {}
  static inline void assume_deinit(RawMut < ::rust::Unit >&) noexcept {}
  static inline size_t size_of() noexcept {
      return 8;
  }
};



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





  
  

  

  
  
  

  
  
  

  
  

namespace rust {

  
    
  
} // namespace rust



  
  

  

  
  
  

  
  
  

  
  

namespace rust {

  
    
  
} // namespace rust



  
  

  

  
  
  

  
  
  

  
  
    
    
    inline ::rust::Unit rust::crate::CharPrinter::print (
      ::rust::Char i0
    ) noexcept {
      ::rust::Unit o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur__crate_CharPrinter_print___x7s13n25m31y32_a4954a65fb (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }

    

    
  
    
    
    inline ::rust::Bool rust::crate::CharPrinter::is_alphabetic (
      ::rust::Char i0
    ) noexcept {
      ::rust::Bool o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur__crate_CharPrinter_is_alphabetic___x7s13n25m39y40_a4954a65fb (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }

    

    
  
    
    
    inline ::rust::Char rust::crate::CharPrinter::to_uppercase (
      ::rust::Char i0
    ) noexcept {
      ::rust::Char o{};
       ::rust::__zngur_internal_assume_deinit(i0); 
      _zngur__crate_CharPrinter_to_uppercase___x7s13n25m38y39_a4954a65fb (
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(o)
      );
      
      ::rust::__zngur_internal_assume_init(o);
      return o;
    }

    

    
  

namespace rust {

  
    
  
} // namespace rust



  
  

  

  
  
  

  
  
  

  
  

namespace rust {

  
    
  
} // namespace rust





namespace rust {
namespace exported_functions {



} // namespace exported_functions



} // namespace rust