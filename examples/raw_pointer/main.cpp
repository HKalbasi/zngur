#include <vector>

#include "./generated.h"

template <typename T>
using Vec = rust::std::vec::Vec<T>;

int main()
{
    Vec<int32_t> s = Vec<int32_t>::new_();
    s.push(2);
    s.push(7);
    zngur_dbg(s);
    s.reserve(3);
    int32_t* s_ptr = s.as_mut_ptr();
    s_ptr[2] = 3;
    s_ptr[3] = 10;
    s_ptr[4] = 2;
    s.set_len(5);
    zngur_dbg(s);

    Vec<Vec<int32_t>> ss = Vec<Vec<int32_t>>::new_();
    ss.push(s.clone());
    ss.push(Vec<int32_t>::new_());
    zngur_dbg(ss);
    ss.reserve(3);
    rust::RawMut<Vec<int32_t>> ss_ptr = ss.as_mut_ptr();
    ss_ptr.offset(2).write(Vec<int32_t>::new_());
    ss_ptr.offset(3).write(s.clone());
    ss_ptr.offset(4).write(s.clone());
    ss.set_len(5);
    zngur_dbg(ss);

    Vec<int32_t> s2 = ss_ptr.offset(4).read();
    ss.set_len(4);

    s2.push(4);
    zngur_dbg(s2);

    zngur_dbg(ss_ptr.read_ref());
    auto s3_ref_mut = ss_ptr.offset(2).read_mut();
    s3_ref_mut.push(2000);
    zngur_dbg(s3_ref_mut);
    zngur_dbg(ss);

    rust::RawMut<Vec<int32_t>> s4_raw_mut = ss.get_mut(2).unwrap();
    s4_raw_mut.read_mut().push(5);
    zngur_dbg(s4_raw_mut.read_ref());
    zngur_dbg(ss);

    rust::Raw<Vec<int32_t>> s4_raw = ss.get_mut(2).unwrap();
    zngur_dbg(s4_raw.read_ref());

    rust::Raw<Vec<int32_t>> ss_ptr2 = ss.as_ptr();
    zngur_dbg(ss_ptr2.offset(2).read_ref());
    zngur_dbg(ss_ptr2.offset(4).offset(-2).read_ref());

    std::vector<int32_t> v { 10, 20, 3, 15 };
    rust::RawMut<rust::Slice<int32_t>> s5_raw_mut { { reinterpret_cast<uint8_t*>(v.data()), 3 } };
    zngur_dbg(s5_raw_mut.read_ref().to_vec());
}
