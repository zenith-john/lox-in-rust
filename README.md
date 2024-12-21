# Lox in Rust

## Introduction

Lox in Rust is an Lox interpreter written in rust following the book [Crafting Interpreters](https://craftinginterpreters.com/).

## How to use?

Clone the repo to local directory and install Rust and cargo. Then run

```shell
cargo run
```

to enter the REPL mode. An example lox file is included in test/test.lox. You can run by

```shell
cargo run test/test.lox
```

to run the script.

## Difference

The author follows the code of [Crafting Interpreters](https://craftinginterpreters.com/) and the function should be the similar to the interpreter in the book with following differences.

- The "For" statement is not implemented.
- The global function environment is not implemented.
- The error message is not unified and quite chaotic in my implementation. And some necessary checks on "this" and "super" statement are omitted.

The detailed differences between implementations can be found in my blog [【井蛙语海】Rust](https://zenith-john.github.io/post/frog_in_well_talking_about_sea_rust/) (only in Chinese though).

## Note

The development of the project is stopped now.