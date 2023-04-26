from time import time

def fib(n):
    if n <= 1:
        return n
    return fib(n - 2) + fib(n - 1)

start = time()
fib40 = fib(40)
stop = time()
print(f"fib(40): {fib40}, time: {stop - start} seconds")
