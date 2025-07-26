# rlox-jasm

JASM IL and JASM Bytecode generating lox compiler written in rust.

Base compiler (lexing, parsing, AST) : [Emirhan Tala](https://github.com/Emivvvvv)  
JASM IL and Bytecode generation      : [Yusuf Ender Osmanoğlu](https://github.com/ysufender)  

This repo is forked from [rlox by Emirhan Tala](https://github.com/Emivvvvv/rlox);

> NOTE:
> Currently the library versions of CSR and JASM are not yet published. And because of that,
> this repo includes both JASM and CSR repos to build and invoke them from rlox-jasm. Sorry
> for the inconvenience.

# rlox-jasm roadmap
IL and Bytecode generation from rlox AST.  

|                       Chapter                        | Status |
|:----------------------------------------------------:|:------:|
|        IL Representations of Basic Operations        |   ⏳   |
|                  Variables on Stack                  |   ⏳   |
|           Statements and jumping around              |   ⏳   |
|                  Function Signatures                 |   ⏳   |
|              Stack and Heap Estimations              |   ⏳   |
|       Non-inheritance Classes (Basic Structs)        |   ⏳   |
|                Inheritance and VTables               |   ⏳   |

### rlox-jasm benchmark

```shell
./run_benchmark.sh
```

| File                 | Took (s)           |
|:--------------------:|:------------------:|
| binary_trees.lox     |        ???         |
| equality.lox         |        ???         |
| fib.lox              |        ???         |
| instantiation.lox    |        ???         |
| invocation.lox       |        ???         |
| method_call.lox      |        ???         |
| properties.lox       |        ???         |
| string_equality.lox  |        ???         |
| trees.lox            |        ???         |
| zoo.lox              |        ???         |

### Building

- Prerequisites
    - GCC and G++ unless you want to configure JASM and CSR preset files under `external/`
    - cargo for, obviously, Rust.
    - make
    - a shell

Command: `make`

That's it. Only problem is it only builds for debug, because I want to debug nowadays.

> Note:
> Do a `make clean` if you want to make a clean build. 

### Specifications

Since rlox was meant to be interpreted, so it can't be directly turned into IL. We need
some specifications. I'll keep updating this part as things come to my mind.

1) All strings are heap objects, in format [u32, char[]] where u32 is lenght.
