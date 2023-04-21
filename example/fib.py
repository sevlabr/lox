from time import time

def fib(n):
    if n <= 1:
        return n
    return fib(n - 2) + fib(n - 1)

start = time()
fib30 = fib(30)
stop = time()
# fib(30): 832040, time: 0.1521151065826416
print(f"fib(30): {fib30}, time: {stop - start}")

start = time()
fib40 = fib(40)
stop = time()
# fib(40): 102334155, time: 16.38774013519287
print(f"fib(40): {fib40}, time: {stop - start}")
