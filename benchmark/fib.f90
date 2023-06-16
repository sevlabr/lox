recursive function fib(n) result(res)
    integer, intent (in) :: n
    integer              :: res
    if (n < 2) then
        res = n
    else
        res = fib(n - 2) + fib(n - 1)
    end if
end function fib

program main
    implicit none
    integer :: fib
    integer :: n
    integer :: res
    real    :: T1, T2
    
    n = 40
    call cpu_time(T1)
    res = fib(n)
    call cpu_time(T2)
    print *, "Result:", res
    print *, "Elapsed in seconds:", T2 - T1
end program main
