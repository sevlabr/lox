#!/bin/bash

# _fix ran with fix in evaluator (+ additional fixes if specified)

cargo run -- example/binding_resolving.lox > example/binding_resolving.out
# cargo run -- example/class_init.lox &> example/class_fix_init.out # ran without silly fix (in .lox)
# cargo run -- example/class.lox > example/class_fix.out
cargo run -- example/class_init.lox > example/class_init.out # ran with silly fix (in .lox)
cargo run -- example/class_super.lox > example/class_super.out # ran with silly fix (in .lox)
cargo run -- example/class.lox &> example/class.out
cargo run -- example/closure.lox > example/closure.out
cargo run -- example/control_flow.lox > example/control_flow.out
cargo run -- example/crazy_function.lox > example/crazy_function.out
cargo run -- example/function.lox > example/function.out
# cargo run -- example/resolver_on_off.lox > example/resolver_off.out
# cargo run -- example/resolver_on_off.lox > example/resolver_on.out
cargo run -- example/semantic.lox &> example/semantic.out
cargo run -- example/var_scope.lox > example/var_scope.out
