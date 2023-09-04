
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <csignal>
#include <array>
#include <iostream>
#include <functional>

#define zngur_dbg(x)                                                           \
  {                                                                            \
    ::std::cerr << "[" << __FILE__ << ":" << __LINE__ << "] " << #x << " = ";  \
    ::rust::zngur_pretty_print(x);                                             \
  }

namespace rust {
    template<typename T>
    uint8_t* __zngur_internal_data_ptr(T& t);

    template<typename T>
    void __zngur_internal_assume_init(T& t);

    template<typename T>
    void __zngur_internal_assume_deinit(T& t);

    template<typename T>
    inline size_t __zngur_internal_size_of();

    template<typename T>
    inline void __zngur_internal_move_to_rust(uint8_t* dst, T& t) {{
        memcpy(dst, ::rust::__zngur_internal_data_ptr(t), ::rust::__zngur_internal_size_of<T>());
        ::rust::__zngur_internal_assume_deinit(t);
    }}

    template<typename T>
    inline T __zngur_internal_move_from_rust(uint8_t* src) {{
        T t;
        ::rust::__zngur_internal_assume_init(t);
        memcpy(::rust::__zngur_internal_data_ptr(t), src, ::rust::__zngur_internal_size_of<T>());
        return t;
    }}

    template<typename T>
    inline void __zngur_internal_check_init(T& t) {{
    }}

    template<typename T>
    struct Ref;

    template<typename T>
    void zngur_pretty_print(T&) {}

    template<typename Type>
    class Impl;

    template<>
    inline uint8_t* __zngur_internal_data_ptr<int8_t>(int8_t& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<int8_t>(int8_t&) {}
    template<>
    inline void __zngur_internal_assume_deinit<int8_t>(int8_t&) {}

    template<>
    inline size_t __zngur_internal_size_of<int8_t>() {
        return sizeof(int8_t);
    }

    template<>
    inline uint8_t* __zngur_internal_data_ptr<int8_t*>(int8_t*& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<int8_t*>(int8_t*&) {}
    template<>
    inline void __zngur_internal_assume_deinit<int8_t*>(int8_t*&) {}

    template<>
    struct Ref<int8_t> {
        Ref() {
            data = 0;
        }
        Ref(int8_t& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }

        int8_t& operator*() {
            return *(int8_t*)data;
        }
        private:
            size_t data;
        friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref<int8_t>>(::rust::Ref<int8_t>& t);
    };


    template<>
    inline uint8_t* __zngur_internal_data_ptr<uint8_t>(uint8_t& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<uint8_t>(uint8_t&) {}
    template<>
    inline void __zngur_internal_assume_deinit<uint8_t>(uint8_t&) {}

    template<>
    inline size_t __zngur_internal_size_of<uint8_t>() {
        return sizeof(uint8_t);
    }

    template<>
    inline uint8_t* __zngur_internal_data_ptr<uint8_t*>(uint8_t*& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<uint8_t*>(uint8_t*&) {}
    template<>
    inline void __zngur_internal_assume_deinit<uint8_t*>(uint8_t*&) {}

    template<>
    struct Ref<uint8_t> {
        Ref() {
            data = 0;
        }
        Ref(uint8_t& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }

        uint8_t& operator*() {
            return *(uint8_t*)data;
        }
        private:
            size_t data;
        friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref<uint8_t>>(::rust::Ref<uint8_t>& t);
    };


    template<>
    inline uint8_t* __zngur_internal_data_ptr<int16_t>(int16_t& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<int16_t>(int16_t&) {}
    template<>
    inline void __zngur_internal_assume_deinit<int16_t>(int16_t&) {}

    template<>
    inline size_t __zngur_internal_size_of<int16_t>() {
        return sizeof(int16_t);
    }

    template<>
    inline uint8_t* __zngur_internal_data_ptr<int16_t*>(int16_t*& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<int16_t*>(int16_t*&) {}
    template<>
    inline void __zngur_internal_assume_deinit<int16_t*>(int16_t*&) {}

    template<>
    struct Ref<int16_t> {
        Ref() {
            data = 0;
        }
        Ref(int16_t& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }

        int16_t& operator*() {
            return *(int16_t*)data;
        }
        private:
            size_t data;
        friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref<int16_t>>(::rust::Ref<int16_t>& t);
    };


    template<>
    inline uint8_t* __zngur_internal_data_ptr<uint16_t>(uint16_t& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<uint16_t>(uint16_t&) {}
    template<>
    inline void __zngur_internal_assume_deinit<uint16_t>(uint16_t&) {}

    template<>
    inline size_t __zngur_internal_size_of<uint16_t>() {
        return sizeof(uint16_t);
    }

    template<>
    inline uint8_t* __zngur_internal_data_ptr<uint16_t*>(uint16_t*& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<uint16_t*>(uint16_t*&) {}
    template<>
    inline void __zngur_internal_assume_deinit<uint16_t*>(uint16_t*&) {}

    template<>
    struct Ref<uint16_t> {
        Ref() {
            data = 0;
        }
        Ref(uint16_t& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }

        uint16_t& operator*() {
            return *(uint16_t*)data;
        }
        private:
            size_t data;
        friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref<uint16_t>>(::rust::Ref<uint16_t>& t);
    };


    template<>
    inline uint8_t* __zngur_internal_data_ptr<int32_t>(int32_t& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<int32_t>(int32_t&) {}
    template<>
    inline void __zngur_internal_assume_deinit<int32_t>(int32_t&) {}

    template<>
    inline size_t __zngur_internal_size_of<int32_t>() {
        return sizeof(int32_t);
    }

    template<>
    inline uint8_t* __zngur_internal_data_ptr<int32_t*>(int32_t*& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<int32_t*>(int32_t*&) {}
    template<>
    inline void __zngur_internal_assume_deinit<int32_t*>(int32_t*&) {}

    template<>
    struct Ref<int32_t> {
        Ref() {
            data = 0;
        }
        Ref(int32_t& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }

        int32_t& operator*() {
            return *(int32_t*)data;
        }
        private:
            size_t data;
        friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref<int32_t>>(::rust::Ref<int32_t>& t);
    };


    template<>
    inline uint8_t* __zngur_internal_data_ptr<uint32_t>(uint32_t& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<uint32_t>(uint32_t&) {}
    template<>
    inline void __zngur_internal_assume_deinit<uint32_t>(uint32_t&) {}

    template<>
    inline size_t __zngur_internal_size_of<uint32_t>() {
        return sizeof(uint32_t);
    }

    template<>
    inline uint8_t* __zngur_internal_data_ptr<uint32_t*>(uint32_t*& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<uint32_t*>(uint32_t*&) {}
    template<>
    inline void __zngur_internal_assume_deinit<uint32_t*>(uint32_t*&) {}

    template<>
    struct Ref<uint32_t> {
        Ref() {
            data = 0;
        }
        Ref(uint32_t& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }

        uint32_t& operator*() {
            return *(uint32_t*)data;
        }
        private:
            size_t data;
        friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref<uint32_t>>(::rust::Ref<uint32_t>& t);
    };


    template<>
    inline uint8_t* __zngur_internal_data_ptr<int64_t>(int64_t& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<int64_t>(int64_t&) {}
    template<>
    inline void __zngur_internal_assume_deinit<int64_t>(int64_t&) {}

    template<>
    inline size_t __zngur_internal_size_of<int64_t>() {
        return sizeof(int64_t);
    }

    template<>
    inline uint8_t* __zngur_internal_data_ptr<int64_t*>(int64_t*& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<int64_t*>(int64_t*&) {}
    template<>
    inline void __zngur_internal_assume_deinit<int64_t*>(int64_t*&) {}

    template<>
    struct Ref<int64_t> {
        Ref() {
            data = 0;
        }
        Ref(int64_t& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }

        int64_t& operator*() {
            return *(int64_t*)data;
        }
        private:
            size_t data;
        friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref<int64_t>>(::rust::Ref<int64_t>& t);
    };


    template<>
    inline uint8_t* __zngur_internal_data_ptr<uint64_t>(uint64_t& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<uint64_t>(uint64_t&) {}
    template<>
    inline void __zngur_internal_assume_deinit<uint64_t>(uint64_t&) {}

    template<>
    inline size_t __zngur_internal_size_of<uint64_t>() {
        return sizeof(uint64_t);
    }

    template<>
    inline uint8_t* __zngur_internal_data_ptr<uint64_t*>(uint64_t*& t) {
        return (uint8_t*)&t;
    }

    template<>
    inline void __zngur_internal_assume_init<uint64_t*>(uint64_t*&) {}
    template<>
    inline void __zngur_internal_assume_deinit<uint64_t*>(uint64_t*&) {}

    template<>
    struct Ref<uint64_t> {
        Ref() {
            data = 0;
        }
        Ref(uint64_t& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }

        uint64_t& operator*() {
            return *(uint64_t*)data;
        }
        private:
            size_t data;
        friend uint8_t* ::rust::__zngur_internal_data_ptr<Ref<uint64_t>>(::rust::Ref<uint64_t>& t);
    };

}
extern "C" {
void __zngur_crate_Reader_drop_in_place_s13e20(uint8_t *data);
void __zngur__crate_Flags_bits___x8s14n20m25y26(uint8_t* i0,uint8_t* o);
void __zngur_crate_Flags_drop_in_place_s13e19(uint8_t *data);
}
namespace rust {
struct Unit;
}
namespace rust {
namespace crate {
struct Reader;
}
}
namespace rust {
namespace crate {
struct Flags;
}
}

namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr<::rust::Unit>(::rust::Unit& t);
    template<>
    inline void __zngur_internal_check_init<::rust::Unit>(::rust::Unit& t);
    template<>
    inline void __zngur_internal_assume_init<::rust::Unit>(::rust::Unit& t);
    template<>
    inline void __zngur_internal_assume_deinit<::rust::Unit>(::rust::Unit& t);
    template<>
    inline size_t __zngur_internal_size_of<::rust::Unit>();
}
namespace rust {
struct Unit
{
private:
    alignas(1) ::std::array<uint8_t, 0> data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<::rust::Unit>(::rust::Unit& t);
    friend void ::rust::__zngur_internal_check_init<::rust::Unit>(::rust::Unit& t);
    friend void ::rust::__zngur_internal_assume_init<::rust::Unit>(::rust::Unit& t);
    friend void ::rust::__zngur_internal_assume_deinit<::rust::Unit>(::rust::Unit& t);
    friend void ::rust::zngur_pretty_print<::rust::Unit>(::rust::Unit& t);

public:
};
}

namespace rust {
    template<>
    inline size_t __zngur_internal_size_of<::rust::Unit>() {
        return 0;
    }


        template<>
        inline void __zngur_internal_check_init<::rust::Unit>(::rust::Unit&) {
        }

        template<>
        inline void __zngur_internal_assume_init<::rust::Unit>(::rust::Unit&) {
        }
    
        template<>
        inline void __zngur_internal_assume_deinit<::rust::Unit>(::rust::Unit&) {
        }


    template<>
    inline uint8_t* __zngur_internal_data_ptr<::rust::Unit>(::rust::Unit& t) {
        ::rust::__zngur_internal_check_init<::rust::Unit>(t);
        return (uint8_t*)&t.data;
    }
}


template<>
struct rust::Ref<::rust::Unit> {
    Ref() {
        data = 0;
    }
    Ref(::rust::Unit& t) {
        data = (size_t)__zngur_internal_data_ptr(t);
    }
private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<::rust::Ref<::rust::Unit>>(::rust::Ref<::rust::Unit>& t);

public:
};

namespace rust {

template<>
inline uint8_t* __zngur_internal_data_ptr<Ref<::rust::Unit>>(Ref<::rust::Unit>& t) {
    return (uint8_t*)&t.data;
}

template<>
inline void __zngur_internal_assume_init<Ref<::rust::Unit>>(Ref<::rust::Unit>&) {
}

template<>
inline void __zngur_internal_check_init<Ref<::rust::Unit>>(Ref<::rust::Unit>&) {
}

template<>
inline void __zngur_internal_assume_deinit<Ref<::rust::Unit>>(Ref<::rust::Unit>&) {
}

template<>
inline size_t __zngur_internal_size_of<Ref<::rust::Unit>>() {
    return 8;
}
}

namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr<::rust::crate::Reader>(::rust::crate::Reader& t);
    template<>
    inline void __zngur_internal_check_init<::rust::crate::Reader>(::rust::crate::Reader& t);
    template<>
    inline void __zngur_internal_assume_init<::rust::crate::Reader>(::rust::crate::Reader& t);
    template<>
    inline void __zngur_internal_assume_deinit<::rust::crate::Reader>(::rust::crate::Reader& t);
    template<>
    inline size_t __zngur_internal_size_of<::rust::crate::Reader>();
}
namespace rust {
namespace crate {
struct Reader
{
private:
    alignas(1) ::std::array<uint8_t, 0> data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<::rust::crate::Reader>(::rust::crate::Reader& t);
    friend void ::rust::__zngur_internal_check_init<::rust::crate::Reader>(::rust::crate::Reader& t);
    friend void ::rust::__zngur_internal_assume_init<::rust::crate::Reader>(::rust::crate::Reader& t);
    friend void ::rust::__zngur_internal_assume_deinit<::rust::crate::Reader>(::rust::crate::Reader& t);
    friend void ::rust::zngur_pretty_print<::rust::crate::Reader>(::rust::crate::Reader& t);

   bool drop_flag;
public:

    Reader() : drop_flag(false) {}
    ~Reader() {
        if (drop_flag) {
            __zngur_crate_Reader_drop_in_place_s13e20(&data[0]);
        }
    }
    Reader(const Reader& other) = delete;
    Reader& operator=(const Reader& other) = delete;
    Reader(Reader&& other) : data(other.data), drop_flag(true) {
        ::rust::__zngur_internal_check_init<Reader>(other);
        other.drop_flag = false;
    }
    Reader& operator=(Reader&& other) {
        *this = Reader(::std::move(other));
        return *this;
    }
    
};
}
}

namespace rust {
    template<>
    inline size_t __zngur_internal_size_of<::rust::crate::Reader>() {
        return 0;
    }


        template<>
        inline void __zngur_internal_check_init<::rust::crate::Reader>(::rust::crate::Reader& t) {
            if (!t.drop_flag) {
                ::std::cerr << "Use of uninitialized or moved Zngur Rust object with type ::rust::crate::Reader" << ::std::endl;
                while (true) raise(SIGSEGV);
            }
        }

        template<>
        inline void __zngur_internal_assume_init<::rust::crate::Reader>(::rust::crate::Reader& t) {
            t.drop_flag = true;
        }
    
        template<>
        inline void __zngur_internal_assume_deinit<::rust::crate::Reader>(::rust::crate::Reader& t) {
            t.drop_flag = false;
        }


    template<>
    inline uint8_t* __zngur_internal_data_ptr<::rust::crate::Reader>(::rust::crate::Reader& t) {
        ::rust::__zngur_internal_check_init<::rust::crate::Reader>(t);
        return (uint8_t*)&t.data;
    }
}


template<>
struct rust::Ref<::rust::crate::Reader> {
    Ref() {
        data = 0;
    }
    Ref(::rust::crate::Reader& t) {
        data = (size_t)__zngur_internal_data_ptr(t);
    }
private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<::rust::Ref<::rust::crate::Reader>>(::rust::Ref<::rust::crate::Reader>& t);

public:
};

namespace rust {

template<>
inline uint8_t* __zngur_internal_data_ptr<Ref<::rust::crate::Reader>>(Ref<::rust::crate::Reader>& t) {
    return (uint8_t*)&t.data;
}

template<>
inline void __zngur_internal_assume_init<Ref<::rust::crate::Reader>>(Ref<::rust::crate::Reader>&) {
}

template<>
inline void __zngur_internal_check_init<Ref<::rust::crate::Reader>>(Ref<::rust::crate::Reader>&) {
}

template<>
inline void __zngur_internal_assume_deinit<Ref<::rust::crate::Reader>>(Ref<::rust::crate::Reader>&) {
}

template<>
inline size_t __zngur_internal_size_of<Ref<::rust::crate::Reader>>() {
    return 8;
}
}

namespace rust {
    template<>
    inline uint8_t* __zngur_internal_data_ptr<::rust::crate::Flags>(::rust::crate::Flags& t);
    template<>
    inline void __zngur_internal_check_init<::rust::crate::Flags>(::rust::crate::Flags& t);
    template<>
    inline void __zngur_internal_assume_init<::rust::crate::Flags>(::rust::crate::Flags& t);
    template<>
    inline void __zngur_internal_assume_deinit<::rust::crate::Flags>(::rust::crate::Flags& t);
    template<>
    inline size_t __zngur_internal_size_of<::rust::crate::Flags>();
}
namespace rust {
namespace crate {
struct Flags
{
private:
    alignas(1) ::std::array<uint8_t, 1> data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<::rust::crate::Flags>(::rust::crate::Flags& t);
    friend void ::rust::__zngur_internal_check_init<::rust::crate::Flags>(::rust::crate::Flags& t);
    friend void ::rust::__zngur_internal_assume_init<::rust::crate::Flags>(::rust::crate::Flags& t);
    friend void ::rust::__zngur_internal_assume_deinit<::rust::crate::Flags>(::rust::crate::Flags& t);
    friend void ::rust::zngur_pretty_print<::rust::crate::Flags>(::rust::crate::Flags& t);

   bool drop_flag;
public:

    Flags() : drop_flag(false) {}
    ~Flags() {
        if (drop_flag) {
            __zngur_crate_Flags_drop_in_place_s13e19(&data[0]);
        }
    }
    Flags(const Flags& other) = delete;
    Flags& operator=(const Flags& other) = delete;
    Flags(Flags&& other) : data(other.data), drop_flag(true) {
        ::rust::__zngur_internal_check_init<Flags>(other);
        other.drop_flag = false;
    }
    Flags& operator=(Flags&& other) {
        *this = Flags(::std::move(other));
        return *this;
    }
    
static ::uint8_t bits(::rust::Ref<::rust::crate::Flags> i0);
::uint8_t bits();
};
}
}

namespace rust {
    template<>
    inline size_t __zngur_internal_size_of<::rust::crate::Flags>() {
        return 1;
    }


        template<>
        inline void __zngur_internal_check_init<::rust::crate::Flags>(::rust::crate::Flags& t) {
            if (!t.drop_flag) {
                ::std::cerr << "Use of uninitialized or moved Zngur Rust object with type ::rust::crate::Flags" << ::std::endl;
                while (true) raise(SIGSEGV);
            }
        }

        template<>
        inline void __zngur_internal_assume_init<::rust::crate::Flags>(::rust::crate::Flags& t) {
            t.drop_flag = true;
        }
    
        template<>
        inline void __zngur_internal_assume_deinit<::rust::crate::Flags>(::rust::crate::Flags& t) {
            t.drop_flag = false;
        }


    template<>
    inline uint8_t* __zngur_internal_data_ptr<::rust::crate::Flags>(::rust::crate::Flags& t) {
        ::rust::__zngur_internal_check_init<::rust::crate::Flags>(t);
        return (uint8_t*)&t.data;
    }
}


template<>
struct rust::Ref<::rust::crate::Flags> {
    Ref() {
        data = 0;
    }
    Ref(::rust::crate::Flags& t) {
        data = (size_t)__zngur_internal_data_ptr(t);
    }
private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr<::rust::Ref<::rust::crate::Flags>>(::rust::Ref<::rust::crate::Flags>& t);

public:
::uint8_t bits();
};

namespace rust {

template<>
inline uint8_t* __zngur_internal_data_ptr<Ref<::rust::crate::Flags>>(Ref<::rust::crate::Flags>& t) {
    return (uint8_t*)&t.data;
}

template<>
inline void __zngur_internal_assume_init<Ref<::rust::crate::Flags>>(Ref<::rust::crate::Flags>&) {
}

template<>
inline void __zngur_internal_check_init<Ref<::rust::crate::Flags>>(Ref<::rust::crate::Flags>&) {
}

template<>
inline void __zngur_internal_assume_deinit<Ref<::rust::crate::Flags>>(Ref<::rust::crate::Flags>&) {
}

template<>
inline size_t __zngur_internal_size_of<Ref<::rust::crate::Flags>>() {
    return 8;
}
}
inline ::uint8_t rust::crate::Flags::bits(::rust::Ref<::rust::crate::Flags> i0)
        {
            ::uint8_t o;
            ::rust::__zngur_internal_assume_init(o);
            __zngur__crate_Flags_bits___x8s14n20m25y26(::rust::__zngur_internal_data_ptr(i0), ::rust::__zngur_internal_data_ptr(o));
            ::rust::__zngur_internal_assume_deinit(i0);
            return o;
        }
inline ::uint8_t rust::Ref<::rust::crate::Flags>::bits()
                {
                    return rust::crate::Flags::bits(*this);
                }
inline ::uint8_t rust::crate::Flags::bits()
                {
                    return rust::crate::Flags::bits(*this);
                }
namespace rust { namespace exported_functions {
   ::rust::crate::Reader new_blob_store_client(::rust::crate::Flags i0);
} }
