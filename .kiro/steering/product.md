# Product Overview

MIR is an experimental intermediate representation (IR) and virtual machine designed for hot-module-reloadable distributed systems. The core innovation is enabling seamless code updates across distributed nodes while preserving application state, connections, and ongoing processes.

## Key Features

- **Hot-Module-Reloading**: Update running code without losing state or breaking connections
- **Distributed Coordination**: Built-in consensus mechanisms for coordinating updates across multiple nodes
- **Content-Addressable Storage**: All code and data are hashed and versioned for precise change detection
- **Schema-Driven Migrations**: Automatic data migration when types evolve
- **Multiple Backends**: Supports WASM, JavaScript/TypeScript, and WASI targets
- **Built-in Observability**: First-class OpenTelemetry integration with automatic instrumentation
- **FFI Integration**: Foreign function interface for host environment interaction

## Target Use Cases

- Development environments requiring fast iteration cycles
- Production systems needing zero-downtime deployments  
- Distributed applications with complex state management
- Systems requiring strong observability and debugging capabilities

The project is currently in experimental phase with active development on the distributed HMR runtime.