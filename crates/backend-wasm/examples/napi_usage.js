// Example usage of WASM NAPI bindings for Node.js
// This demonstrates the comprehensive JavaScript interop capabilities

const { 
    WasmNapiBindings, 
    WasmInstanceManager, 
    WasmMemoryManager, 
    FFIBridge,
    ValueConverter 
} = require('../index.js');

async function demonstrateWasmNapiBindings() {
    console.log('=== WASM NAPI Bindings Demo ===\n');

    // 1. Initialize NAPI bindings
    const napiBindings = new WasmNapiBindings();
    console.log('✓ NAPI bindings initialized');

    // 2. Compile MIR module to WASM
    const mirModule = {
        name: "example_module",
        statements: [
            {
                type: "FunctionDeclaration",
                name: "add",
                parameters: ["a", "b"],
                body: [
                    {
                        type: "Expression",
                        expression: {
                            type: "FunctionCall",
                            function: { type: "Identifier", name: "add_impl" },
                            arguments: [
                                { type: "Identifier", name: "a" },
                                { type: "Identifier", name: "b" }
                            ]
                        }
                    }
                ]
            }
        ]
    };

    try {
        const wasmModule = await napiBindings.compileModule(JSON.stringify(mirModule));
        console.log('✓ MIR module compiled to WASM');
        console.log(`  - Exports: ${wasmModule.exports.length}`);
        console.log(`  - Functions: ${wasmModule.functions.length}`);
        console.log(`  - Has memory: ${wasmModule.hasMemory}`);

        // 3. Instance Management
        const instanceManager = new WasmInstanceManager({
            maxInstances: 10,
            memoryLimitMb: 64,
            executionTimeoutMs: 5000,
            enableHotReload: true,
            preserveStateOnReload: true,
            enableDebugging: true
        });

        const instanceId = "example_instance";
        const createResult = await instanceManager.createInstance(
            instanceId,
            wasmModule.bytecode,
            null // no imports for this example
        );

        if (createResult.success) {
            console.log('✓ WASM instance created successfully');
            console.log(`  - Memory usage: ${createResult.memoryDeltaBytes} bytes`);
            console.log(`  - Creation time: ${createResult.durationMs.toFixed(2)}ms`);

            // 4. Memory Management
            const memoryManager = new WasmMemoryManager();
            
            // Allocate some memory
            const allocResult = memoryManager.allocateMemory(instanceId, 1024, "buffer");
            if (allocResult.success) {
                console.log('✓ Memory allocated');
                console.log(`  - Allocation ID: ${allocResult.allocationId}`);
                console.log(`  - Offset: ${allocResult.offset}`);
                console.log(`  - Size: ${allocResult.size} bytes`);

                // Write data to memory
                const testData = Buffer.from("Hello, WASM!", "utf8");
                const writeResult = memoryManager.writeMemory(instanceId, allocResult.offset, testData);
                if (writeResult.success) {
                    console.log('✓ Data written to WASM memory');

                    // Read data back
                    const readData = memoryManager.readMemory(instanceId, allocResult.offset, testData.length);
                    console.log(`✓ Data read from memory: "${readData.toString('utf8')}"`);
                }

                // Get memory statistics
                const memStats = memoryManager.getMemoryStats(instanceId);
                console.log('✓ Memory statistics:');
                console.log(`  - Total pages: ${memStats.totalPages}`);
                console.log(`  - Used pages: ${memStats.usedPages}`);
                console.log(`  - Active allocations: ${memStats.activeAllocations}`);
                console.log(`  - Fragmentation ratio: ${(memStats.fragmentationRatio * 100).toFixed(2)}%`);
            }

            // 5. FFI Bridge
            const ffiBridge = new FFIBridge();

            // Register host function
            const hostFunctionConfig = {
                name: "console_log",
                bindingType: "function",
                isAsync: false,
                paramTypes: ["string"],
                resultTypes: [],
                preserveState: false
            };

            const hostFunction = (message) => {
                console.log(`[WASM]: ${message}`);
                return null;
            };

            ffiBridge.registerHostFunction(hostFunctionConfig, hostFunction);
            console.log('✓ Host function registered');

            // Call host function from JavaScript (simulating WASM call)
            const callResult = await ffiBridge.callHostFunction(
                "console_log",
                ["Hello from WASM!"],
                instanceId
            );

            if (callResult.success) {
                console.log('✓ Host function called successfully');
                console.log(`  - Execution time: ${callResult.executionTimeMs.toFixed(2)}ms`);
            }

            // 6. Value Conversion
            const valueConverter = new ValueConverter();

            // Convert JavaScript values to WASM
            const jsNumber = 42.5;
            const wasmI32 = valueConverter.jsToWasm(jsNumber, "i32");
            console.log('✓ JavaScript number converted to WASM i32');

            const jsString = "Hello, World!";
            const wasmString = valueConverter.jsToWasm(jsString, "string");
            console.log('✓ JavaScript string converted to WASM string');

            // Batch conversion
            const batchRequest = {
                values: [42, "test", true, [1, 2, 3]],
                sourceTypes: ["number", "string", "boolean", "array"],
                targetTypes: ["i32", "string", "i32", "array"],
                conversionOptions: {
                    strictTyping: false,
                    allowLossyConversion: true,
                    useCache: true,
                    validateRanges: false,
                    preservePrecision: false
                }
            };

            const batchResult = valueConverter.batchConvert(batchRequest);
            console.log('✓ Batch conversion completed');
            console.log(`  - Success rate: ${(batchResult.successRate * 100).toFixed(2)}%`);
            console.log(`  - Total time: ${batchResult.totalTimeMs.toFixed(2)}ms`);

            // 7. Hot-Reload Demonstration
            console.log('\n--- Hot-Reload Demo ---');
            
            // Simulate module update
            const updatedMirModule = {
                ...mirModule,
                statements: [
                    ...mirModule.statements,
                    {
                        type: "FunctionDeclaration",
                        name: "multiply",
                        parameters: ["x", "y"],
                        body: []
                    }
                ]
            };

            const updatedWasmModule = await napiBindings.compileModule(JSON.stringify(updatedMirModule));
            const hotReloadResult = await instanceManager.hotReloadInstance(
                instanceId,
                updatedWasmModule.bytecode
            );

            if (hotReloadResult.success) {
                console.log('✓ Hot-reload completed successfully');
                console.log(`  - State preserved: ${hotReloadResult.statePreserved}`);
                console.log(`  - FFI bindings preserved: ${hotReloadResult.ffiBindingsPreserved}`);
                console.log(`  - Affected exports: ${hotReloadResult.affectedExports.join(', ')}`);
                console.log(`  - Reload time: ${hotReloadResult.durationMs.toFixed(2)}ms`);
            }

            // 8. Statistics and Monitoring
            console.log('\n--- Statistics ---');
            
            const instanceStats = instanceManager.getInstanceStats(instanceId);
            console.log('Instance Statistics:');
            console.log(`  - Memory usage: ${instanceStats.memoryUsageBytes} bytes`);
            console.log(`  - Execution count: ${instanceStats.executionCount}`);
            console.log(`  - Hot-reload count: ${instanceStats.hotReloadCount}`);
            console.log(`  - Last execution: ${instanceStats.lastExecutionTime}`);

            const ffiStats = ffiBridge.getFfiStats();
            console.log('FFI Statistics:');
            console.log(`  - Host bindings: ${ffiStats.totalHostBindings}`);
            console.log(`  - WASM exports: ${ffiStats.totalWasmExports}`);
            console.log(`  - Total calls: ${ffiStats.totalCalls}`);
            console.log(`  - Success rate: ${((ffiStats.successfulCalls / ffiStats.totalCalls) * 100).toFixed(2)}%`);

            const conversionStats = valueConverter.getConversionStats();
            console.log('Conversion Statistics:');
            console.log(`  - Total conversions: ${conversionStats.totalConversions}`);
            console.log(`  - Cache hit rate: ${((conversionStats.cacheHits / (conversionStats.cacheHits + conversionStats.cacheMisses)) * 100).toFixed(2)}%`);

            // 9. Cleanup
            const removed = instanceManager.removeInstance(instanceId);
            console.log(`✓ Instance cleanup: ${removed ? 'success' : 'failed'}`);

        } else {
            console.error('✗ Failed to create WASM instance:', createResult.error);
        }

    } catch (error) {
        console.error('✗ Error during demo:', error.message);
    }
}

// Advanced usage examples
async function advancedUsageExamples() {
    console.log('\n=== Advanced Usage Examples ===\n');

    const napiBindings = new WasmNapiBindings();
    
    // 1. Custom optimization levels
    napiBindings.setOptimizationLevel("aggressive");
    console.log('✓ Optimization level set to aggressive');

    // 2. Memory management with typed arrays
    const memoryManager = new WasmMemoryManager();
    const instanceId = "advanced_instance";
    
    // Create memory view for direct access
    const memoryView = memoryManager.createMemoryView(instanceId, 0, 1024, "uint8");
    console.log('✓ Memory view created for direct access');

    // 3. FFI with async callbacks
    const ffiBridge = new FFIBridge();
    
    const asyncHostFunction = async (data) => {
        // Simulate async operation
        await new Promise(resolve => setTimeout(resolve, 100));
        return { processed: data, timestamp: Date.now() };
    };

    const asyncConfig = {
        name: "async_processor",
        bindingType: "async_callback",
        isAsync: true,
        paramTypes: ["object"],
        resultTypes: ["object"],
        preserveState: true
    };

    ffiBridge.registerHostFunction(asyncConfig, asyncHostFunction);
    console.log('✓ Async host function registered');

    // 4. State preservation during hot-reload
    const preservationResult = ffiBridge.preserveFfiState(instanceId);
    if (preservationResult.success) {
        console.log('✓ FFI state preserved');
        console.log(`  - Preserved bindings: ${preservationResult.preservedBindings.length}`);
        console.log(`  - State size: ${preservationResult.preservedStateSize} bytes`);
    }

    // 5. Custom type mappings
    const valueConverter = new ValueConverter();
    
    valueConverter.registerTypeMapping(
        "custom_date",
        "object",
        "i64",
        "i64",
        "custom:date_to_timestamp"
    );
    console.log('✓ Custom type mapping registered');

    // 6. Memory compaction and garbage collection
    const compactionResult = memoryManager.compactMemory(instanceId);
    if (compactionResult.success) {
        console.log('✓ Memory compacted');
    }

    const gcCount = memoryManager.garbageCollect(instanceId);
    console.log(`✓ Garbage collection: ${gcCount} allocations cleaned up`);
}

// Error handling examples
async function errorHandlingExamples() {
    console.log('\n=== Error Handling Examples ===\n');

    const napiBindings = new WasmNapiBindings();
    
    try {
        // Invalid MIR module
        await napiBindings.compileModule("invalid json");
    } catch (error) {
        console.log('✓ Invalid MIR module error handled:', error.message);
    }

    const instanceManager = new WasmInstanceManager();
    
    try {
        // Non-existent instance
        await instanceManager.executeFunction("non_existent", "test_function", []);
    } catch (error) {
        console.log('✓ Non-existent instance error handled:', error.message);
    }

    const memoryManager = new WasmMemoryManager();
    
    try {
        // Memory bounds violation
        memoryManager.readMemory("test_instance", 999999, 1024);
    } catch (error) {
        console.log('✓ Memory bounds error handled:', error.message);
    }
}

// Run all examples
async function runAllExamples() {
    try {
        await demonstrateWasmNapiBindings();
        await advancedUsageExamples();
        await errorHandlingExamples();
        
        console.log('\n=== Demo Complete ===');
        console.log('All NAPI binding features demonstrated successfully!');
        
    } catch (error) {
        console.error('Demo failed:', error);
        process.exit(1);
    }
}

// Export for use as module
module.exports = {
    demonstrateWasmNapiBindings,
    advancedUsageExamples,
    errorHandlingExamples,
    runAllExamples
};

// Run if called directly
if (require.main === module) {
    runAllExamples();
}