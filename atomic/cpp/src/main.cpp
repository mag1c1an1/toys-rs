#include <atomic>
#include <iostream>
#include <thread>

bool one() {
  auto y = std::atomic<int>(0);
  auto x = std::atomic<int>(0);

  int f1 = 1, f2 = 2;

  std::thread j1([&] {
    auto r1 = y.load(std::memory_order_relaxed);
    x.store(r1, std::memory_order_relaxed);
    f1 = r1;
    return;
  });
  std::thread j2([&] {
    auto r2 = x.load(std::memory_order_relaxed);
    y.store(42, std::memory_order_relaxed);
    f2 = r2;
    return;
  });
  j1.join();
  j2.join();
  return f1 == f2 && f1 == 42;
}

int main(int argc, char **argv) {
  while (!one()) {
  }
  std::cout << "atomic is truth" << std::endl;
  return 0;
}
