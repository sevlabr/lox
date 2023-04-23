#!/bin/bash

cargo run -- example/binding_resolving.lox > example/binding_resolving.out
cargo run -- example/closure.lox > example/closure.out
cargo run -- example/control_flow.lox > example/control_flow.out
cargo run -- example/crazy_function.lox > example/crazy_function.out
cargo run -- example/function.lox > example/function.out
# cargo run -- example/resolver_on_off.lox > example/resolver_off.out
# cargo run -- example/resolver_on_off.lox > example/resolver_on.out
cargo run -- example/semantic.lox &> example/semantic.out
cargo run -- example/var_scope.lox > example/var_scope.out
