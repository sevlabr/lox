use std::time::Instant;

fn fib(n: usize) -> usize {
    if n < 2 {
        return n;
    }
    fib(n - 2) + fib(n - 1)
}

fn main() {
    let start = Instant::now();
    let n = fib(40);
    let duration = start.elapsed();

    println!("Time elapsed in fib(40) is: {:?}, result is: {}", duration, n);
}
