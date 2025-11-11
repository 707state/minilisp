#include <doctest.h>

#include <iomanip>
#include <iostream>

#include "common.hpp"

TEST_CASE("bitmask imm encoding test")
{
  struct Test {
    uint64_t v;
  };
  Test tests[] = {
      {1ULL},
      {2ULL},
      {3ULL},
      {4ULL},
      {0x5555555555555555ULL},
      {0xF0F0F0F0F0F0F0F0ULL},
      {0x0FF00FF00FF00FF0ULL},
      {0xFFULL},
      {0xFFFFFFFFFFFFFFFEULL}
      // should be invalid if all ones? actually only u64::MAX invalid earlier
  };

  for (auto &t : tests) {
    BitmaskImmediate b;
    bool ok = BitmaskImmediate::try_from(t.v, b);
    CHECK_EQ(ok, true);
    std::cout << "value=0x" << std::hex << std::setw(16) << std::setfill('0')
              << t.v << std::dec << " -> ";
    if (!ok) {
      std::cout << "NOT encodable\n";
    }
    else {
      std::cout << "n=" << static_cast<int>(b.n) << " imms=0x" << std::hex
                << static_cast<int>(b.imms) << " immr=0x"
                << static_cast<int>(b.immr) << " enc=0x" << std::hex
                << b.to_u32() << std::dec << "\n";
    }
  }
}
