#include <doctest.h>
template <typename T> struct type_identity {
  using type = T;
};
template <typename T> using type_identity_t = typename type_identity<T>::type;
template <typename U> U implicit_cast(typename type_identity<U>::type val) { return val; }

class Base {};
class Derived : public Base {};

template <typename T> bool foo(const T &a, const T &b) {
    return true;
}

TEST_CASE("implicit_cast"){
  Base base;
  Derived derived;
  CHECK_EQ(foo(base, implicit_cast<Base&>(derived)),true);
}
