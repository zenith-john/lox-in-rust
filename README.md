# Lox in Rust

## Introduction

Lox in Rust is an Lox interpreter written in rust following the book [Crafting Interpreters](https://craftinginterpreters.com/). Both the interpreter model and virtual machine model.

## How to use?

Clone the repo to local directory and install Rust and cargo. Then run

```shell
cargo build
```

to build the interpreter. Then you can use

```shell
cargo run
```

to enter the REPL mode, or
```shell
cargo run FILENAME
```
to execute the script file.

Some examples of lox file is included in test. You can run by

```shell
cargo run test
```

to run the script.

## Note

The function of interpreted is complete except for statement. I'm satisfied with the current form and will not revisit this program in the near future.
