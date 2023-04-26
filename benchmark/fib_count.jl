using BenchmarkTools: @btime

mutable struct Counter
    n::Int
end

"Count number of function calls."
function gen_fib()
    counter = Counter(0)

    function fib(n)
        counter.n += 1
        if n < 2
            return n
        end
        return fib(n - 2) + fib(n - 1)
    end

    return fib, counter
end

fib, counter = gen_fib()
n = 40
res = fib(n)
num_func_calls = counter.n
println("Result: $res, number of function calls: $num_func_calls\nTime elapsed for this closure function:")
fib, _ = gen_fib()
@btime fib($40)
