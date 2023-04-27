#!/bin/bash

# _fix ran with fix in evaluator (+ additional fixes if specified)

cargo run -p twi -- example/debug/binding_resolving.lox > example/debug/binding_resolving.out
# cargo run -p twi -- example/debug/class_init.lox &> example/debug/class_fix_init.out # ran without silly fix (in .lox)
# cargo run -p twi -- example/debug/class.lox > example/debug/class_fix.out
cargo run -p twi -- example/debug/class_init.lox > example/debug/class_init.out # ran with silly fix (in .lox)
cargo run -p twi -- example/debug/class_super.lox > example/debug/class_super.out # ran with silly fix (in .lox)
cargo run -p twi -- example/debug/class.lox &> example/debug/class.out
cargo run -p twi -- example/debug/closure.lox > example/debug/closure.out
cargo run -p twi -- example/debug/control_flow.lox > example/debug/control_flow.out
cargo run -p twi -- example/debug/crazy_function.lox > example/debug/crazy_function.out
cargo run -p twi -- example/debug/function.lox > example/debug/function.out
# cargo run -p twi -- example/debug/resolver_on_off.lox > example/debug/resolver_off.out
# cargo run -p twi -- example/debug/resolver_on_off.lox > example/debug/resolver_on.out
cargo run -p twi -- example/debug/semantic.lox &> example/debug/semantic.out
cargo run -p twi -- example/debug/var_scope.lox > example/debug/var_scope.out
