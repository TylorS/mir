# Project Structure

## Workspace Organization

The project follows a multi-crate Cargo workspace pattern with clear separation of concerns:

```
crates/
├── ast/                    # Abstract Syntax Tree definitions
├── backend-js/             # JavaScript/TypeScript backend
│   └── src/typescript/     # TypeScript-specific code generation
├── backend-wasi/           # WASI backend implementation
│   └── src/napi/          # NAPI bindings for WASI
├── backend-wasm/           # WebAssembly backend
│   └── src/napi/          # NAPI bindings for WASM
├── compiler/               # Core compilation logic
│   ├── src/linker/        # Module linking and dependency resolution
│   └── src/optimizer/     # Compile-time optimizations
├── napi/                   # Node.js API bindings
├── runtime/                # Virtual machine runtime
├── types/                  # Core type system definitions
└── vm/                     # Virtual machine implementation
    └── src/hot-module-reloading/  # HMR-specific logic
```

## Core Components

### Frontend (AST & Types)
- **`ast/`**: Defines the intermediate representation structure
- **`types/`**: Core type system with RTTI support

### Compilation Pipeline
- **`compiler/`**: Main compilation orchestration
  - **`linker/`**: Module dependency resolution and linking
  - **`optimizer/`**: Dead code elimination, constant folding, etc.

### Runtime System
- **`runtime/`**: Core runtime services and coordination
- **`vm/`**: Virtual machine execution engine
  - **`hot-module-reloading/`**: State preservation and update coordination

### Backend Targets
- **`backend-wasm/`**: WebAssembly code generation
- **`backend-js/`**: JavaScript/TypeScript code generation
- **`backend-wasi/`**: WASI component model support
- **`napi/`**: Node.js native bindings

## Architectural Principles

- **Separation of Concerns**: Each crate has a single, well-defined responsibility
- **Backend Agnostic**: Core logic is independent of target platform
- **Modular Design**: Components can be developed and tested independently
- **Content Addressable**: All modules and data are identified by content hash
- **Schema Evolution**: Types serve as versioned schemas for automatic migration

## Development Guidelines

- Keep crate interfaces minimal and well-documented
- Use workspace dependencies for shared functionality
- Maintain clear boundaries between compilation phases
- Design for hot-reloadability from the ground up
- Prioritize observability and debugging support