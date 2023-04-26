# C
gcc -o "fib-c-unopt" fib.c
./fib-c-unopt
gcc -o "fib-c-O3" -O3 fib.c
./fib-c-O3
gcc -o "fib-c-O3-ffast-math" -O3 -ffast-math fib.c
./fib-c-O3-ffast-math

# C++
g++ -o "fib-cpp-unopt" fib.cpp
./fib-cpp-unopt
g++ -o "fib-cpp-O3" -O3 fib.cpp
./fib-cpp-O3
g++ -o "fib-cpp-O3-ffast-math" -O3 -ffast-math fib.cpp
./fib-cpp-O3-ffast-math

# Go
go run fib.go

# Haskell
ghc -o "fib-hs-unopt" fib.hs
time ./fib-hs-unopt
ghc -o "fib-hs-O2" -O2 fib.hs
time ./fib-hs-O2

# Julia
julia fib.jl
julia -O3 fib.jl
julia -O3 --handle-signals=no --min-optlevel=3 -g0 --inline=yes --check-bounds=no --math-mode=fast --compile=all fib.jl
julia -O3 fib_count.jl

# Python
python fib.py

# Rust
rustc -o "fib-rs-unopt" fib.rs
./fib-rs-unopt
rustc -O -o "fib-rs-opt" fib.rs
./fib-rs-opt
