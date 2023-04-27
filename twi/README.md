# What is it?

*Rust* implementation of *jlox* variant of *Lox* language as it is described in a
magnificent book
[Crafting Interpreters](https://craftinginterpreters.com/).

# Structure

- `/src` — code for tree-walk interpreter
- `/example` — simple example programs on *Lox* language
- `/example/debug` — programs with different edge cases on *Lox* that I used for testing (run tests via `run_example.sh`)

# Does it work?

For simple programs it does. But closures and sometimes classes may work really weird or even give an error for a valid code.
The reason for this is how I implemented environments. The original *Java* implementation stores a reference of an environment
for closure. When I was writing this part of interpreter I thought that it would be easier for me to use copies and update them.
It turned out to be wrong. I guess, usual approach would be the use of `Rc<RefCell<Environment>>`, so probably I will fix this problem
with environments if I have time. Overall, lexer, parser and resolver seem to work just fine, only the interpreter (which I call
*evaluator* in code) has some bugs.

# Examples

All basic and human-readable examples are in `/example` folder
(`debug` subfolder contains some tests for edge cases and thus the code here is probably quite hard to understand).
Currently, there are 3 examples: classes, closures and basic functions. *.lox* files are the source code
and *.out* files contain outputs.

TODO

# AST visualization

TODO
