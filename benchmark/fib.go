package main

import (
	"log"
	"time"
)

func fib(n uint) uint {
	if n < 2 {
		return n
	}
	return fib(n-2) + fib(n-1)
}

func main() {
	start := time.Now()
	n := fib(40)
	elapsed := time.Since(start)
	log.Printf("fib(40) is %d and took %s", n, elapsed)
}
