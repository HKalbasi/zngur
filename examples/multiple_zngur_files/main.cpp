#include <iostream>

#include "./generated1.h"
#undef zngur_dbg
#include "./generated2.h"

template <typename T>
using Vec1 = m1::std::vec::Vec<T>;
template <typename T>
using Vec2 = m2::std::vec::Vec<T>;

int main()
{
    auto s1 = Vec1<int32_t>::new_();
    s1.push(2);
    Vec1<int32_t>::push(s1, 5);
    s1.push(7);
    Vec1<int32_t>::push(s1, 3);
    std::cout << *s1.get(2).unwrap() << std::endl;
    std::cout << s1.clone().into_iter().sum() << std::endl;

    auto s2 = Vec2<int32_t>::new_();
    s2.push(2);
    Vec2<int32_t>::push(s2, 5);
    s2.push(7);
    Vec2<int32_t>::push(s2, 3);
    std::cout << *s2.get(1).unwrap() << std::endl;
}
