# rjavavm

`rjavavm` is a small experimental Java VM written in Rust.
It can load Java class files, print parsed class information, and execute a limited subset of JVM bytecode.

## Capabilities

- Load a `.class` file and print its structure
- Run a class by name
- Trace executed bytecode operations with `--trace-ops`

## Implemented

- Basic class file loading and parsing
- Constant pool, fields, methods, and attributes parsing
- Runtime with stack frames, local variables, and operand stack
- A small set of bytecode instructions, including:
  `sipush`, `bipush`, `iload`, `iload_0..3`, `if*`, `if_icmp*`, `getstatic`, `invokestatic`, `invokedynamic`, and `return`
- Basic method and field resolution

## Not Implemented

- Most JVM instructions
- Full object model and heap-based object instances
- Arrays
- Exception handling
- Virtual dispatch and complete method invocation support
- Garbage collection
- Full Java standard library support

## Usage

`cargo run -- --print test/Hello.class`

`cargo run -- --run Hello`

`cargo run -- --run Hello --trace-ops`

When using `cargo run`, keep the extra `--` before program arguments.