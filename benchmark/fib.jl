using BenchmarkTools: @btime

function fib(n)
    if n < 2
        return n
    end
    return fib(n - 2) + fib(n - 1)
end

begin
    println("Time:")
    @btime fib($40)
    res = fib(40)
    println("Result: $res")
end
