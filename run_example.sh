#!/bin/bash

cargo run -- example/var_scope.lox > example/var_scope.out
cargo run -- example/binding_resolving.lox > example/binding_resolving.out
cargo run -- example/control_flow.lox > example/control_flow.out
cargo run -- example/function.lox > example/function.out
cargo run -- example/crazy_function.lox > example/crazy_function.out
cargo run -- example/closure.lox > example/closure.out
