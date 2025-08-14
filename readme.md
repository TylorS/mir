# MIR 

MIR is an experimentatal IR for a hot-module-reloadable VM for higher-level languages targeting 
WASM, with GC supported structs and arrays, with first-class support for FFI to varying hosts.

All MIR is hashed and serialized to a content addressable store. This allows for hot-module-reloading
with state preservation, spinning up new instances of modules and other dynamic distributed systems.

All infrastructure is built as code and can be configured to scale to any number of nodes from development
to production. Choreography is built in.

## Features

### Intermediate Representation

- Module system with FFI
- First class functions
- Strict type system with RTTI
- Data structures (structs, arrays, unions, resources)
- Built-in telemetry

### Virtual Machine

- Hot-module-reloading w/ state preservation, change detection and diffing
- FFI
- Strict type system with RTTI, casting, and type-level reflection
- Built-in hashing and serialization of ALL of the language
- Built-in metrics, logging, and telemetry
- Built-in typed databases with migrations as code
- Built-in RPC system with configurable transports and storage
- Built-in durable workflow orchestration
- Built-in auto-scaling, load balancing, and fault-tolerance

### Multiple backends

- WASM + optional NAPI binding for node.js
- JS/TS + WASM
- WASI + Component model

## Linker + Optimizer 

- Compile time optimizations
- Dead code elimination
- Constant folding
- Copy propagation
- Dead store elimination
- Inlining
- Loop unrolling
- Loop vectorization
- Loop parallelization
- Loop fusion
- Loop tiling
