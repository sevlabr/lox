#include <stdio.h>
#include <time.h>

unsigned int fib(unsigned int n) {
    if (n < 2) {
        return n;
    }
    return fib(n - 2) + fib(n - 1);
}

int main() {
    unsigned int n = 40;
    unsigned int res;
    clock_t t;
    t = clock();
    res = fib(n);
    t = clock() - t;
    double time_taken = ((double)t)/(CLOCKS_PER_SEC/1000);
    printf("fib(40): %d\nTook %f milliseconds to execute\n", res, time_taken);
}
