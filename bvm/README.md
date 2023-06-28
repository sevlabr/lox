# What is it?

*Rust* implementation of *clox* variant of *Lox* language as it is described in a
brilliant book
[Crafting Interpreters](https://craftinginterpreters.com/).

# Structure

- `/src` — code for bytecode virtual machine

# Does it work?

Right now `bvm` is implemented up to *Garbage Collection* chapter.
So it does not support GC and OOP yet. But all other features work correctly, as far as I know.

Currently, my Rust implementation is slower than the original one that is written in C.
The difference varies significantly from one benchmark to another. Usually, `bvm` is about
2-4 times slower than `clox`. But if you call a lot of functions the efficiency can drop by 2 orders
of magnitude.

# Examples

`bvm` can not only execute source code but also print:

- output from lexer
- bytecode
- current stack state during evaluation

For more info use `-h` option.

For example, consider this simple program that prints *11111*:

```
var one = "1";
var ones = "";

for (var i = 1; i < 10; i = i + 2) {
    ones = ones + one;
}

print ones;
```

Executed with `-s` option:

```
   1          Var 'var'
   |   Identifier 'one'
   |        Equal '='
   |       String '"1"'
   |    Semicolon ';'
   2          Var 'var'
   |   Identifier 'ones'
   |        Equal '='
   |       String '""'
   |    Semicolon ';'
   4          For 'for'
   |    LeftParen '('
   |          Var 'var'
   |   Identifier 'i'
   |        Equal '='
   |       Number '1'
   |    Semicolon ';'
   |   Identifier 'i'
   |         Less '<'
   |       Number '10'
   |    Semicolon ';'
   |   Identifier 'i'
   |        Equal '='
   |   Identifier 'i'
   |         Plus '+'
   |       Number '2'
   |   RightParen ')'
   |    LeftBrace '{'
   5   Identifier 'ones'
   |        Equal '='
   |   Identifier 'ones'
   |         Plus '+'
   |   Identifier 'one'
   |    Semicolon ';'
   6   RightBrace '}'
   8        Print 'print'
   |   Identifier 'ones'
   |    Semicolon ';'
   9          EoF ''
```

Executed with `-b` option:

```
== code ==
0000    1 OP_CONSTANT         1 '1'
0002    | OP_DEFINE_GLOBAL    0 'one'
0004    2 OP_CONSTANT         3 ''
0006    | OP_DEFINE_GLOBAL    2 'ones'
0008    4 OP_CONSTANT         4 '1'
0010    | OP_GET_LOCAL        1
0012    | OP_CONSTANT         5 '10'
0014    | OP_LESS
0015    | OP_JUMP_IF_FALSE   15 -> 44
0018    | OP_POP
0019    | OP_JUMP            19 -> 33
0022    | OP_GET_LOCAL        1
0024    | OP_CONSTANT         6 '2'
0026    | OP_ADD
0027    | OP_SET_LOCAL        1
0029    | OP_POP
0030    | OP_LOOP            30 -> 10
0033    5 OP_GET_GLOBAL       8 'ones'
0035    | OP_GET_GLOBAL       9 'one'
0037    | OP_ADD
0038    | OP_SET_GLOBAL       7 'ones'
0040    | OP_POP
0041    6 OP_LOOP            41 -> 22
0044    | OP_POP
0045    | OP_POP
0046    8 OP_GET_GLOBAL      10 'ones'
0048    | OP_PRINT
0049    9 OP_NIL
0050    | OP_RETURN
```

Executed with `-t` option:

```
          [ <script> ]
0000    1 OP_CONSTANT         1 '1'
          [ <script> ][ 1 ]
0002    | OP_DEFINE_GLOBAL    0 'one'
          [ <script> ]
0004    2 OP_CONSTANT         3 ''
          [ <script> ][  ]
0006    | OP_DEFINE_GLOBAL    2 'ones'
          [ <script> ]
0008    4 OP_CONSTANT         4 '1'
          [ <script> ][ 1 ]
0010    | OP_GET_LOCAL        1
          [ <script> ][ 1 ][ 1 ]
0012    | OP_CONSTANT         5 '10'
          [ <script> ][ 1 ][ 1 ][ 10 ]
0014    | OP_LESS
          [ <script> ][ 1 ][ true ]
0015    | OP_JUMP_IF_FALSE   15 -> 44
          [ <script> ][ 1 ][ true ]
0018    | OP_POP
          [ <script> ][ 1 ]
0019    | OP_JUMP            19 -> 33
          [ <script> ][ 1 ]
0033    5 OP_GET_GLOBAL       8 'ones'
          [ <script> ][ 1 ][  ]
0035    | OP_GET_GLOBAL       9 'one'
          [ <script> ][ 1 ][  ][ 1 ]
0037    | OP_ADD
          [ <script> ][ 1 ][ 1 ]
0038    | OP_SET_GLOBAL       7 'ones'
          [ <script> ][ 1 ][ 1 ]
0040    | OP_POP
          [ <script> ][ 1 ]
0041    6 OP_LOOP            41 -> 22
          [ <script> ][ 1 ]
0022    | OP_GET_LOCAL        1
          [ <script> ][ 1 ][ 1 ]
0024    | OP_CONSTANT         6 '2'
          [ <script> ][ 1 ][ 1 ][ 2 ]
0026    | OP_ADD
          [ <script> ][ 1 ][ 3 ]
0027    | OP_SET_LOCAL        1
          [ <script> ][ 3 ][ 3 ]
0029    | OP_POP
          [ <script> ][ 3 ]
0030    | OP_LOOP            30 -> 10
          [ <script> ][ 3 ]
0010    | OP_GET_LOCAL        1
          [ <script> ][ 3 ][ 3 ]
0012    | OP_CONSTANT         5 '10'
          [ <script> ][ 3 ][ 3 ][ 10 ]
0014    | OP_LESS
          [ <script> ][ 3 ][ true ]
0015    | OP_JUMP_IF_FALSE   15 -> 44
          [ <script> ][ 3 ][ true ]
0018    | OP_POP
          [ <script> ][ 3 ]
0019    | OP_JUMP            19 -> 33
          [ <script> ][ 3 ]
0033    5 OP_GET_GLOBAL       8 'ones'
          [ <script> ][ 3 ][ 1 ]
0035    | OP_GET_GLOBAL       9 'one'
          [ <script> ][ 3 ][ 1 ][ 1 ]
0037    | OP_ADD
          [ <script> ][ 3 ][ 11 ]
0038    | OP_SET_GLOBAL       7 'ones'
          [ <script> ][ 3 ][ 11 ]
0040    | OP_POP
          [ <script> ][ 3 ]
0041    6 OP_LOOP            41 -> 22
          [ <script> ][ 3 ]
0022    | OP_GET_LOCAL        1
          [ <script> ][ 3 ][ 3 ]
0024    | OP_CONSTANT         6 '2'
          [ <script> ][ 3 ][ 3 ][ 2 ]
0026    | OP_ADD
          [ <script> ][ 3 ][ 5 ]
0027    | OP_SET_LOCAL        1
          [ <script> ][ 5 ][ 5 ]
0029    | OP_POP
          [ <script> ][ 5 ]
0030    | OP_LOOP            30 -> 10
          [ <script> ][ 5 ]
0010    | OP_GET_LOCAL        1
          [ <script> ][ 5 ][ 5 ]
0012    | OP_CONSTANT         5 '10'
          [ <script> ][ 5 ][ 5 ][ 10 ]
0014    | OP_LESS
          [ <script> ][ 5 ][ true ]
0015    | OP_JUMP_IF_FALSE   15 -> 44
          [ <script> ][ 5 ][ true ]
0018    | OP_POP
          [ <script> ][ 5 ]
0019    | OP_JUMP            19 -> 33
          [ <script> ][ 5 ]
0033    5 OP_GET_GLOBAL       8 'ones'
          [ <script> ][ 5 ][ 11 ]
0035    | OP_GET_GLOBAL       9 'one'
          [ <script> ][ 5 ][ 11 ][ 1 ]
0037    | OP_ADD
          [ <script> ][ 5 ][ 111 ]
0038    | OP_SET_GLOBAL       7 'ones'
          [ <script> ][ 5 ][ 111 ]
0040    | OP_POP
          [ <script> ][ 5 ]
0041    6 OP_LOOP            41 -> 22
          [ <script> ][ 5 ]
0022    | OP_GET_LOCAL        1
          [ <script> ][ 5 ][ 5 ]
0024    | OP_CONSTANT         6 '2'
          [ <script> ][ 5 ][ 5 ][ 2 ]
0026    | OP_ADD
          [ <script> ][ 5 ][ 7 ]
0027    | OP_SET_LOCAL        1
          [ <script> ][ 7 ][ 7 ]
0029    | OP_POP
          [ <script> ][ 7 ]
0030    | OP_LOOP            30 -> 10
          [ <script> ][ 7 ]
0010    | OP_GET_LOCAL        1
          [ <script> ][ 7 ][ 7 ]
0012    | OP_CONSTANT         5 '10'
          [ <script> ][ 7 ][ 7 ][ 10 ]
0014    | OP_LESS
          [ <script> ][ 7 ][ true ]
0015    | OP_JUMP_IF_FALSE   15 -> 44
          [ <script> ][ 7 ][ true ]
0018    | OP_POP
          [ <script> ][ 7 ]
0019    | OP_JUMP            19 -> 33
          [ <script> ][ 7 ]
0033    5 OP_GET_GLOBAL       8 'ones'
          [ <script> ][ 7 ][ 111 ]
0035    | OP_GET_GLOBAL       9 'one'
          [ <script> ][ 7 ][ 111 ][ 1 ]
0037    | OP_ADD
          [ <script> ][ 7 ][ 1111 ]
0038    | OP_SET_GLOBAL       7 'ones'
          [ <script> ][ 7 ][ 1111 ]
0040    | OP_POP
          [ <script> ][ 7 ]
0041    6 OP_LOOP            41 -> 22
          [ <script> ][ 7 ]
0022    | OP_GET_LOCAL        1
          [ <script> ][ 7 ][ 7 ]
0024    | OP_CONSTANT         6 '2'
          [ <script> ][ 7 ][ 7 ][ 2 ]
0026    | OP_ADD
          [ <script> ][ 7 ][ 9 ]
0027    | OP_SET_LOCAL        1
          [ <script> ][ 9 ][ 9 ]
0029    | OP_POP
          [ <script> ][ 9 ]
0030    | OP_LOOP            30 -> 10
          [ <script> ][ 9 ]
0010    | OP_GET_LOCAL        1
          [ <script> ][ 9 ][ 9 ]
0012    | OP_CONSTANT         5 '10'
          [ <script> ][ 9 ][ 9 ][ 10 ]
0014    | OP_LESS
          [ <script> ][ 9 ][ true ]
0015    | OP_JUMP_IF_FALSE   15 -> 44
          [ <script> ][ 9 ][ true ]
0018    | OP_POP
          [ <script> ][ 9 ]
0019    | OP_JUMP            19 -> 33
          [ <script> ][ 9 ]
0033    5 OP_GET_GLOBAL       8 'ones'
          [ <script> ][ 9 ][ 1111 ]
0035    | OP_GET_GLOBAL       9 'one'
          [ <script> ][ 9 ][ 1111 ][ 1 ]
0037    | OP_ADD
          [ <script> ][ 9 ][ 11111 ]
0038    | OP_SET_GLOBAL       7 'ones'
          [ <script> ][ 9 ][ 11111 ]
0040    | OP_POP
          [ <script> ][ 9 ]
0041    6 OP_LOOP            41 -> 22
          [ <script> ][ 9 ]
0022    | OP_GET_LOCAL        1
          [ <script> ][ 9 ][ 9 ]
0024    | OP_CONSTANT         6 '2'
          [ <script> ][ 9 ][ 9 ][ 2 ]
0026    | OP_ADD
          [ <script> ][ 9 ][ 11 ]
0027    | OP_SET_LOCAL        1
          [ <script> ][ 11 ][ 11 ]
0029    | OP_POP
          [ <script> ][ 11 ]
0030    | OP_LOOP            30 -> 10
          [ <script> ][ 11 ]
0010    | OP_GET_LOCAL        1
          [ <script> ][ 11 ][ 11 ]
0012    | OP_CONSTANT         5 '10'
          [ <script> ][ 11 ][ 11 ][ 10 ]
0014    | OP_LESS
          [ <script> ][ 11 ][ false ]
0015    | OP_JUMP_IF_FALSE   15 -> 44
          [ <script> ][ 11 ][ false ]
0044    | OP_POP
          [ <script> ][ 11 ]
0045    | OP_POP
          [ <script> ]
0046    8 OP_GET_GLOBAL      10 'ones'
          [ <script> ][ 11111 ]
0048    | OP_PRINT
11111
          [ <script> ]
0049    9 OP_NIL
          [ <script> ][ nil ]
0050    | OP_RETURN
```
