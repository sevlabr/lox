#include <chrono>
#include <iostream>

using namespace std;
 
unsigned fib(unsigned n) {
    if (n <= 1)
        return n;
    return fib(n - 2) + fib(n - 1);
}

int main() {
    unsigned n;
    auto start = std::chrono::high_resolution_clock::now();
    n = fib(40);
    auto stop = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(stop - start);
    cout << "Result: " << n << endl;
    cout << "Elapsed: " << duration.count() << " milliseconds" << endl;
}
