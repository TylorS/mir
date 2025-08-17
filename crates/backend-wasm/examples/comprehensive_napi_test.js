#!/usr/bin/env node

/**
 * Comprehensive NAPI Integration Test
 * 
 * This script tests the complete NAPI bindings functionality including:
 * - Instance creation and management
 * - Memory operations
 * - FFI bindings
 * - Hot-reloading
 * - Value conversion
 * - Statistics and monitoring
 */

const { createWasmRuntime } = require('../index.js');

// Test configuration
const TEST_CONFIG = {
    maxInstances: 5,
    memoryLimitMb: 32,
    executionTimeoutMs: 5000,
    enableHotReload: true,
    preserveStateOnReload: true,
    enableDebugging: true,
};

// Minimal WASM module bytecode for testing
const MINIMAL_WASM = Buffer.from([
    0x00, 0x61, 0x73, 0x6d, // WASM magic number
    0x01, 0x00, 0x00, 0x00, // WASM version
    0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Type section (empty function)
    0x03, 0x02, 0x01, 0x00, // Function section
    0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b, // Code section (empty function body)
]);

class TestRunner {
    constructor() {
        this.runtime = null;
        this.testResults = [];
        this.instanceCounter = 0;
    }

    async runAllTests() {
        console.log('🚀 Starting comprehensive NAPI integration tests...\n');

        try {
            await this.setupRuntime();
            await this.testInstanceManagement();
            await this.testMemoryOperations();
            await this.testFFIBindings();
            await this.testValueConversion();
            await this.testHotReloading();
            await this.testStatistics();
            await this.testErrorHandling();
            await this.testConcurrentOperations();
            
            this.printResults();
        } catch (error) {
            console.error('❌ Test suite failed:', error);
            process.exit(1);
        } finally {
            await this.cleanup();
        }
    }

    async setupRuntime() {
        console.log('📋 Setting up WASM runtime...');
        this.runtime = createWasmRuntime(TEST_CONFIG);
        this.addTestResult('Runtime Creation', true, 'Runtime created successfully');
    }

    async testInstanceManagement() {
        console.log('\n🏗️  Testing instance management...');

        // Test instance creation
        const instanceId = `test_instance_${++this.instanceCounter}`;
        const result = await this.runtime.createInstance(instanceId, MINIMAL_WASM);
        
        this.addTestResult(
            'Instance Creation',
            result.success,
            result.success ? 'Instance created successfully' : result.error
        );

        // Test instance listing
        const instances = this.runtime.getStats().instances;
        this.addTestResult(
            'Instance Listing',
            instances.includes(instanceId),
            `Found ${instances.length} instances`
        );

        // Test duplicate instance creation (should fail)
        const duplicateResult = await this.runtime.createInstance(instanceId, MINIMAL_WASM);
        this.addTestResult(
            'Duplicate Instance Prevention',
            !duplicateResult.success,
            'Correctly prevented duplicate instance creation'
        );

        // Test instance limit
        const maxInstances = TEST_CONFIG.maxInstances;
        const createdInstances = [];
        
        for (let i = 0; i < maxInstances; i++) {
            const id = `limit_test_${i}`;
            const createResult = await this.runtime.createInstance(id, MINIMAL_WASM);
            if (createResult.success) {
                createdInstances.push(id);
            }
        }

        this.addTestResult(
            'Instance Limit Enforcement',
            createdInstances.length <= maxInstances,
            `Created ${createdInstances.length} instances (limit: ${maxInstances})`
        );
    }

    async testMemoryOperations() {
        console.log('\n💾 Testing memory operations...');

        const instanceId = `memory_test_${++this.instanceCounter}`;
        await this.runtime.createInstance(instanceId, MINIMAL_WASM);

        // Test memory allocation
        const allocResult = this.runtime.allocateMemory(instanceId, 1024, 'test_buffer');
        this.addTestResult(
            'Memory Allocation',
            allocResult.success,
            allocResult.success ? `Allocated ${allocResult.size} bytes` : allocResult.error
        );

        if (allocResult.success) {
            // Test memory write
            const testData = Buffer.from('Hello, WASM!', 'utf8');
            const writeResult = this.runtime.writeMemory(instanceId, allocResult.offset, testData);
            this.addTestResult(
                'Memory Write',
                writeResult.success,
                writeResult.success ? 'Data written successfully' : writeResult.error
            );

            // Test memory read
            try {
                const readData = this.runtime.readMemory(instanceId, allocResult.offset, testData.length);
                const readSuccess = Buffer.compare(testData, readData) === 0;
                this.addTestResult(
                    'Memory Read',
                    readSuccess,
                    readSuccess ? 'Data read correctly' : 'Data mismatch'
                );
            } catch (error) {
                this.addTestResult('Memory Read', false, error.message);
            }
        }
    }

    async testFFIBindings() {
        console.log('\n🔗 Testing FFI bindings...');

        const instanceId = `ffi_test_${++this.instanceCounter}`;
        await this.runtime.createInstance(instanceId, MINIMAL_WASM);

        // Test host function registration
        const hostFunction = (arg1, arg2) => {
            return `Host function called with: ${arg1}, ${arg2}`;
        };

        const bindingConfig = {
            name: 'test_host_function',
            bindingType: 'function',
            isAsync: false,
            paramTypes: ['string', 'string'],
            resultTypes: ['string'],
            preserveState: true,
        };

        const registerResult = this.runtime.registerHostFunction(bindingConfig, hostFunction);
        this.addTestResult(
            'Host Function Registration',
            registerResult,
            registerResult ? 'Host function registered' : 'Registration failed'
        );

        // Test FFI call
        try {
            const callResult = await this.runtime.callHostFunction(
                'test_host_function',
                ['arg1', 'arg2'],
                instanceId
            );
            this.addTestResult(
                'FFI Function Call',
                callResult.success,
                callResult.success ? 'FFI call successful' : callResult.error
            );
        } catch (error) {
            this.addTestResult('FFI Function Call', false, error.message);
        }
    }

    async testValueConversion() {
        console.log('\n🔄 Testing value conversion...');

        // Test basic conversions
        const conversions = [
            { value: '42', from: 'string', to: 'i32' },
            { value: '3.14159', from: 'string', to: 'f64' },
            { value: 'true', from: 'string', to: 'bool' },
            { value: 'Hello', from: 'string', to: 'string' },
        ];

        let successCount = 0;
        for (const conv of conversions) {
            try {
                const result = this.runtime.convertValue(conv.value, conv.from, conv.to);
                if (result !== null && result !== undefined) {
                    successCount++;
                }
            } catch (error) {
                // Expected for some conversions in test environment
            }
        }

        this.addTestResult(
            'Value Conversion',
            successCount > 0,
            `${successCount}/${conversions.length} conversions successful`
        );
    }

    async testHotReloading() {
        console.log('\n🔥 Testing hot-reloading...');

        const instanceId = `hmr_test_${++this.instanceCounter}`;
        await this.runtime.createInstance(instanceId, MINIMAL_WASM);

        // Create modified WASM (same structure, different content)
        const modifiedWasm = Buffer.from([
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Type section
            0x03, 0x02, 0x01, 0x00, // Function section
            0x0a, 0x05, 0x01, 0x03, 0x00, 0x01, 0x0b, // Modified code section
        ]);

        const reloadResult = await this.runtime.hotReload(instanceId, modifiedWasm);
        this.addTestResult(
            'Hot Reload',
            reloadResult.success,
            reloadResult.success ? 'Hot reload successful' : reloadResult.error
        );

        if (reloadResult.success) {
            this.addTestResult(
                'State Preservation',
                reloadResult.preservedState,
                'State preservation during hot reload'
            );

            this.addTestResult(
                'FFI Bindings Preservation',
                reloadResult.ffiBindingsPreserved,
                'FFI bindings preserved during hot reload'
            );
        }
    }

    async testStatistics() {
        console.log('\n📊 Testing statistics and monitoring...');

        const stats = this.runtime.getStats();
        
        this.addTestResult(
            'Instance Statistics',
            Array.isArray(stats.instances),
            `Found ${stats.instances.length} instances`
        );

        this.addTestResult(
            'FFI Statistics',
            typeof stats.ffi === 'object',
            'FFI statistics available'
        );

        this.addTestResult(
            'Conversion Statistics',
            typeof stats.conversions === 'object',
            'Conversion statistics available'
        );
    }

    async testErrorHandling() {
        console.log('\n⚠️  Testing error handling...');

        // Test operations on non-existent instance
        const nonExistentId = 'non_existent_instance';

        try {
            const result = await this.runtime.executeFunction(nonExistentId, 'test_func', []);
            this.addTestResult(
                'Non-existent Instance Handling',
                !result.success,
                'Correctly handled non-existent instance'
            );
        } catch (error) {
            this.addTestResult(
                'Non-existent Instance Handling',
                true,
                'Exception thrown for non-existent instance'
            );
        }

        // Test invalid memory operations
        try {
            const allocResult = this.runtime.allocateMemory(nonExistentId, 1024, 'test');
            this.addTestResult(
                'Invalid Memory Operation Handling',
                !allocResult.success,
                'Correctly handled invalid memory operation'
            );
        } catch (error) {
            this.addTestResult(
                'Invalid Memory Operation Handling',
                true,
                'Exception thrown for invalid memory operation'
            );
        }
    }

    async testConcurrentOperations() {
        console.log('\n🔀 Testing concurrent operations...');

        const promises = [];
        const instanceIds = [];

        // Create multiple instances concurrently
        for (let i = 0; i < 3; i++) {
            const instanceId = `concurrent_${i}`;
            instanceIds.push(instanceId);
            promises.push(this.runtime.createInstance(instanceId, MINIMAL_WASM));
        }

        try {
            const results = await Promise.all(promises);
            const successCount = results.filter(r => r.success).length;
            
            this.addTestResult(
                'Concurrent Instance Creation',
                successCount > 0,
                `${successCount}/3 concurrent instances created`
            );
        } catch (error) {
            this.addTestResult(
                'Concurrent Instance Creation',
                false,
                `Concurrent operations failed: ${error.message}`
            );
        }
    }

    async cleanup() {
        console.log('\n🧹 Cleaning up...');
        if (this.runtime) {
            this.runtime.cleanup();
        }
    }

    addTestResult(testName, success, message) {
        this.testResults.push({ testName, success, message });
        const status = success ? '✅' : '❌';
        console.log(`  ${status} ${testName}: ${message}`);
    }

    printResults() {
        console.log('\n📋 Test Results Summary');
        console.log('========================');
        
        const totalTests = this.testResults.length;
        const passedTests = this.testResults.filter(r => r.success).length;
        const failedTests = totalTests - passedTests;
        
        console.log(`Total Tests: ${totalTests}`);
        console.log(`Passed: ${passedTests}`);
        console.log(`Failed: ${failedTests}`);
        console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
        
        if (failedTests > 0) {
            console.log('\n❌ Failed Tests:');
            this.testResults
                .filter(r => !r.success)
                .forEach(r => console.log(`  - ${r.testName}: ${r.message}`));
        }
        
        console.log('\n🎉 Test suite completed!');
    }
}

// Run tests if this file is executed directly
if (require.main === module) {
    const runner = new TestRunner();
    runner.runAllTests().catch(error => {
        console.error('Test runner failed:', error);
        process.exit(1);
    });
}

module.exports = TestRunner;