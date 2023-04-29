#!/bin/bash

names=("class" "closure" "fib" "simple")

for name in ${names[@]}; do
    lox_path="example/$name.lox"
    out_path="example/$name.out"
    gv_path="gallery/$name.gv"
    svg_path="gallery/$name.svg"

    cargo run --release -p twi -- $lox_path > $out_path
    cargo run --release -p twi -- $lox_path -v > $gv_path
    dot -Tsvg $gv_path -o $svg_path
done
