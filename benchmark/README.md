# What is it?

A set of simple scripts on different languages that compute the 40th Fibonacci number using
naive recursive approach. This single benchmark can't say much about performance of each of the
languages and defenetely can't be used to speculate on topics such as "best" or "fastest" of whatever language.
Rather, the purpose here is to get the idea on how production-ready languages handle function calls and recursion
and compare my *Lox* implementations with these languages. As a bonus I checked how efficient optimizations
can be by compiling the same script for each language with different options. Also, note that I ran
`fib()` functions just one single time in most cases (except *Julia*), so the results presented here can differ
somewhat notably from one run to another. So again, this is not a real language benchmarks.

Bash script that does the whole work is `compile_run.sh`. Raw results are in `result.out`.

PC specs: Intel(R) Core(TM) i7-6700HQ CPU @ 2.60GHz (with CPU max GHz: 3.5) and 16 Gb RAM.

**Note**: to calculate the 40th Fibonacci number you need to call `fib()` function *331'160'281* times.
Since optimized versions of C/C++ and Rust take roughly 0.3 seconds to calculate it, seems like they
don't really call any functions but sort of "elide" them (not sure if I can call it "inlining").
Otherwise, I have no idea how to implement an honest function call that works in just 1 nanosecond.

# Results

Some kind of a summary of `result.out` contents. The 2nd and the 3rd columns is execution time in seconds.
*OFF* means no optimizations, *ON* means best result with optimizations.

**Note**: in fact, tree-walk variant of *Lox* that I wrote is so slow, that I report results here based not on a
real evaluation, but on estimation (see details in ...).

| Language | OFF | ON |
|---|---|---|
| C | 0.6 | 0.3 |
| C++ | 0.6 | 0.3 |
| Go | 0.5 | - |
| Haskell | 9 | 0.5 |
| Julia | 0.5 | 0.5 |
| Lox (bvm) | - | - |
| Lox (twi) | \\( \sim 10^5 \\) | \\( \sim 10^4 \\) |
| Python | 15 | - |
| Rust | 1 | 0.3 |
