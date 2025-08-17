# WASM Backend with NAPI Bindings

This crate provides comprehensive WebAssembly compilation and Node.js integration for the MIR runtime, with full support for hot-module-reloading and FFI integration.

## Features

### Core Capabilities
- **WASM Code Generation**: Compile MIR IR to optimized WebAssembly bytecode
- **Node.js NAPI Bindings**: Seamless JavaScript interop with WASM modules
- **Hot-Module-Reloading**: Update running WASM code while preserving state
- **Memory Management**: Advanced memory allocation and garbage collection
- **FFI Bridge**: Foreign function interface for host environment integration
- **Value Conversion**: Type-safe conversion between JavaScript and WASM values

### Advanced Features
- **State Preservation**: Maintain application state during hot-reloads
- **FFI Binding Preservation**: Keep host connections intact across updates
- **Memory Optimization**: Automatic compaction and fragmentation management
- **Async Support**: Full support for asynchronous operations
- **Debugging Integration**: Source maps and debugging information preservation
- **Performance Monitoring**: Comprehensive statistics and telemetry

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    NAPI Bindings Layer                     │
├─────────────────────────────────────────────────────────────┤
│  WasmNapiBindings  │  InstanceManager  │  MemoryManager   │
│                    │                   │                   │
│  FFIBridge         │  ValueConverter   │  JsWasmConverter │
├─────────────────────────────────────────────────────────────┤
│                    WASM Runtime Layer                      │
├─────────────────────────────────────────────────────────────┤
│  WasmCodeGenerator │  WasmOptimizer    │  Wasmtime Engine │
├─────────────────────────────────────────────────────────────┤
│                    MIR Integration                         │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Installation

Add to your `package.json`:

```json
{
  "dependencies": {
    "mir-wasm-backend": "^0.1.0"
  }
}
```

### Basic Usage

```javascript
const { WasmNapiBindings } = require('mir-wasm-backend');

async function example() {
    // Initialize NAPI bindings
    const bindings = new WasmNapiBindings();
    
    // Compile MIR module to WASM
    const mirModule = {
        name: "example",
        statements: [/* MIR statements */]
    };
    
    const wasmModule = await bindings.compileModule(JSON.stringify(mirModule));
    
    // Instantiate WASM module
    const result = await bindings.instantiateModule(
        "my_instance",
        wasmModule.bytecode,
        null // no imports
    );
    
    if (result.success) {
        console.log("WASM module instantiated successfully!");
    }
}
```

## API Reference

### WasmNapiBindings

Main entry point for WASM compilation and instantiation.

#### Methods

- `compileModule(moduleJson: string): Promise<JsWasmModule>`
- `instantiateModule(instanceId: string, bytecode: Buffer, imports?: JsObject): Promise<JsExecutionResult>`
- `callFunction(instanceId: string, functionName: string, args: Value[]): Promise<JsExecutionResult>`
- `hotReloadModule(instanceId: string, newBytecode: Buffer, preserveState?: boolean): Promise<JsHotReloadResult>`
- `getMemoryStats(instanceId: string): JsMemoryStats`
- `setOptimizationLevel(level: string): void`

### WasmInstanceManager

Advanced instance lifecycle management.

#### Configuration

```javascript
const config = {
    maxInstances: 100,
    memoryLimitMb: 512,
    executionTimeoutMs: 30000,
    enableHotReload: true,
    preserveStateOnReload: true,
    enableDebugging: false
};

const manager = new WasmInstanceManager(config);
```

#### Methods

- `createInstance(instanceId: string, bytecode: Buffer, imports?: JsObject): Promise<InstanceOperationResult>`
- `executeFunction(instanceId: string, functionName: string, args: Value[]): Promise<InstanceOperationResult>`
- `hotReloadInstance(instanceId: string, newBytecode: Buffer): Promise<HotReloadResult>`
- `getInstanceStats(instanceId: string): InstanceStats`
- `removeInstance(instanceId: string): boolean`

### WasmMemoryManager

Comprehensive memory management for WASM instances.

#### Methods

- `registerMemory(instanceId: string, memory: External<Memory>): boolean`
- `allocateMemory(instanceId: string, size: number, allocationType: string): MemoryOperationResult`
- `deallocateMemory(instanceId: string, allocationId: string): MemoryOperationResult`
- `readMemory(instanceId: string, offset: number, size: number): Buffer`
- `writeMemory(instanceId: string, offset: number, data: Buffer): MemoryOperationResult`
- `getMemoryStats(instanceId: string): MemoryStats`
- `compactMemory(instanceId: string): MemoryOperationResult`
- `garbageCollect(instanceId: string): number`

### FFIBridge

Foreign function interface for host environment integration.

#### Host Function Registration

```javascript
const ffiBridge = new FFIBridge();

const config = {
    name: "console_log",
    bindingType: "function",
    isAsync: false,
    paramTypes: ["string"],
    resultTypes: [],
    preserveState: false
};

const hostFunction = (message) => {
    console.log(`[WASM]: ${message}`);
};

ffiBridge.registerHostFunction(config, hostFunction);
```

#### Methods

- `registerHostFunction(config: HostBindingConfig, function: JsFunction): boolean`
- `callHostFunction(functionName: string, args: Value[], instanceId: string): Promise<FFICallResult>`
- `callWasmFunction(instanceId: string, functionName: string, args: Value[]): Promise<FFICallResult>`
- `preserveFfiState(instanceId: string): StatePreservationResult`
- `restoreFfiState(instanceId: string): StatePreservationResult`
- `getFfiStats(): FFIBridgeStats`

### ValueConverter

Type-safe conversion between JavaScript and WASM values.

#### Methods

- `jsToWasm(jsValue: JsUnknown, targetType: string): ConversionResult`
- `wasmToJs(wasmValue: Value, targetType: string): JsUnknown`
- `mirToJs(mirValue: string): JsUnknown`
- `jsToMir(jsValue: JsUnknown): string`
- `batchConvert(request: BatchConversionRequest): BatchConversionResult`
- `registerTypeMapping(name: string, jsType: string, wasmType: string, mirType: string, strategy: string): boolean`

## Hot-Module-Reloading

The NAPI bindings provide comprehensive hot-reloading support that preserves:

- **Application State**: In-memory data structures and variables
- **FFI Bindings**: Host function connections and callbacks
- **Memory Allocations**: Active memory regions and their contents
- **Execution Context**: Call stacks and local variables (where possible)

### Hot-Reload Process

1. **State Preservation**: Extract current state from running instance
2. **FFI Binding Backup**: Save host function connections
3. **New Instance Creation**: Compile and instantiate updated code
4. **State Restoration**: Apply preserved state to new instance
5. **FFI Reconnection**: Restore host function bindings
6. **Validation**: Verify successful update and rollback if needed

### Example

```javascript
// Initial module
const wasmModule = await bindings.compileModule(JSON.stringify(originalModule));
await bindings.instantiateModule("app", wasmModule.bytecode);

// ... application runs ...

// Update module
const updatedModule = await bindings.compileModule(JSON.stringify(newModule));
const reloadResult = await bindings.hotReloadModule("app", updatedModule.bytecode, true);

if (reloadResult.success) {
    console.log("Hot-reload successful!");
    console.log(`State preserved: ${reloadResult.preservedState}`);
    console.log(`FFI bindings preserved: ${reloadResult.ffiBindingsPreserved}`);
} else {
    console.error("Hot-reload failed:", reloadResult.error);
}
```

## Memory Management

### Allocation Strategies

- **First-Fit**: Simple and fast allocation
- **Best-Fit**: Minimize fragmentation
- **Buddy System**: Efficient for power-of-2 sizes
- **Custom**: User-defined allocation strategies

### Memory Views

Create typed views for direct memory access:

```javascript
const memoryManager = new WasmMemoryManager();

// Create a Uint8Array view
const view = memoryManager.createMemoryView("instance", 0, 1024, "uint8");

// Read/write typed data
const data = view.readTypedArray(0, 256);
view.writeTypedArray(0, new Uint8Array([1, 2, 3, 4]));
```

### Garbage Collection

Automatic cleanup of inactive allocations:

```javascript
// Manual garbage collection
const freedAllocations = memoryManager.garbageCollect("instance");
console.log(`Freed ${freedAllocations} allocations`);

// Memory compaction
const compactionResult = memoryManager.compactMemory("instance");
if (compactionResult.success) {
    console.log("Memory compacted successfully");
}
```

## FFI Integration

### Host Function Types

- **Synchronous Functions**: Direct JavaScript function calls
- **Asynchronous Functions**: Promise-based operations
- **Event Handlers**: Callback-based event processing
- **Memory Accessors**: Direct memory manipulation
- **Global Variables**: Shared state management

### State Preservation

FFI bindings can preserve state across hot-reloads:

```javascript
const config = {
    name: "stateful_function",
    bindingType: "function",
    isAsync: false,
    paramTypes: ["object"],
    resultTypes: ["object"],
    preserveState: true  // Enable state preservation
};

let internalState = { counter: 0 };

const statefulFunction = (input) => {
    internalState.counter++;
    return { ...input, callCount: internalState.counter };
};

ffiBridge.registerHostFunction(config, statefulFunction);
```

## Performance Optimization

### Compilation Optimization

```javascript
// Set optimization level
bindings.setOptimizationLevel("aggressive");

// Available levels: "none", "size", "speed", "aggressive"
```

### Memory Optimization

- **Allocation Pooling**: Reuse memory blocks
- **Lazy Allocation**: Allocate on first use
- **Compaction**: Reduce fragmentation
- **Garbage Collection**: Automatic cleanup

### Caching

- **Compilation Cache**: Reuse compiled modules
- **Conversion Cache**: Cache type conversions
- **Instance Cache**: Pool instance creation

## Error Handling

### Error Types

- **Compilation Errors**: Invalid MIR or WASM generation failures
- **Runtime Errors**: Execution failures and traps
- **Memory Errors**: Out-of-bounds access and allocation failures
- **FFI Errors**: Host function call failures
- **Hot-Reload Errors**: State preservation and restoration failures

### Error Recovery

```javascript
try {
    const result = await bindings.callFunction("instance", "risky_function", []);
    if (!result.success) {
        console.error("Function call failed:", result.error);
        // Implement recovery logic
    }
} catch (error) {
    console.error("Unexpected error:", error);
    // Implement fallback behavior
}
```

## Debugging and Monitoring

### Statistics Collection

```javascript
// Instance statistics
const instanceStats = manager.getInstanceStats("my_instance");
console.log(`Memory usage: ${instanceStats.memoryUsageBytes} bytes`);
console.log(`Execution count: ${instanceStats.executionCount}`);

// FFI statistics
const ffiStats = ffiBridge.getFfiStats();
console.log(`Success rate: ${(ffiStats.successfulCalls / ffiStats.totalCalls * 100).toFixed(2)}%`);

// Memory statistics
const memStats = memoryManager.getMemoryStats("my_instance");
console.log(`Fragmentation: ${(memStats.fragmentationRatio * 100).toFixed(2)}%`);
```

### Debug Information

Enable debugging for detailed information:

```javascript
const config = {
    enableDebugging: true,
    // ... other options
};

const manager = new WasmInstanceManager(config);
```

## Examples

See the `examples/` directory for comprehensive usage examples:

- `napi_usage.js`: Complete demonstration of all features
- `hot_reload_demo.js`: Hot-reloading scenarios
- `memory_management.js`: Advanced memory management
- `ffi_integration.js`: FFI bridge usage
- `performance_optimization.js`: Optimization techniques

## Requirements

- Node.js 16+ with NAPI 8 support
- Rust 1.70+ for compilation
- WebAssembly support in target environment

## Building

```bash
# Build the NAPI bindings
cargo build --release

# Run tests
cargo test

# Build Node.js module
npm run build

# Run JavaScript examples
node examples/napi_usage.js
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.