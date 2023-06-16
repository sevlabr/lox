# C
echo "--- C ---"
gcc -o "fib-c-unopt" fib.c
./fib-c-unopt
gcc -o "fib-c-O3" -O3 fib.c
./fib-c-O3
gcc -o "fib-c-O3-ffast-math" -O3 -ffast-math fib.c
./fib-c-O3-ffast-math
echo "----------"
echo ""

# C++
echo "--- C++ ---"
g++ -o "fib-cpp-unopt" fib.cpp
./fib-cpp-unopt
g++ -o "fib-cpp-O3" -O3 fib.cpp
./fib-cpp-O3
g++ -o "fib-cpp-O3-ffast-math" -O3 -ffast-math fib.cpp
./fib-cpp-O3-ffast-math
echo "----------"
echo ""

# Fortran
echo "--- Fortran ---"
gfortran -o "fib-fort-unopt" fib.f90
./fib-fort-unopt
gfortran -o "fib-fort-opt" -O3 -ffast-math fib.f90
./fib-fort-opt
echo "----------"
echo ""

# Go
echo "--- Go ---"
go run fib.go
echo "----------"
echo ""

# Haskell
echo "--- Haskell ---"
ghc -o "fib-hs-unopt" fib.hs
time ./fib-hs-unopt
ghc -o "fib-hs-O2" -O2 fib.hs
time ./fib-hs-O2
echo "----------"
echo ""

# Julia
echo "--- Julia ---"
julia fib.jl
julia -O3 fib.jl
julia -O3 --handle-signals=no --min-optlevel=3 -g0 --inline=yes --check-bounds=no --math-mode=fast --compile=all fib.jl
echo ""
echo "--- Fib count ---"
julia -O3 fib_count.jl
echo "----------"
echo ""

# Lox
# cargo run -p twi -- ./../twi/example/fib.lox
# cargo run --release -p twi -- ./../twi/example/fib.lox

# Python
echo "--- Python ---"
python fib.py
echo "----------"
echo ""

# Rust
echo "--- Rust ---"
rustc -o "fib-rs-unopt" fib.rs
./fib-rs-unopt
rustc -O -o "fib-rs-opt" fib.rs
./fib-rs-opt
echo "----------"
echo ""

# Zig
echo "--- Zig ---"
zig build-exe --name "fib-zig-unopt" fib.zig
./fib-zig-unopt
zig build-exe --name "fib-zig-opt" -O ReleaseFast fib.zig
./fib-zig-opt
echo "----------"
echo ""
