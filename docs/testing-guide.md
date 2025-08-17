# Testing Guide for MIR Runtime

This document provides comprehensive guidance on testing the MIR runtime system, including unit tests, integration tests, and end-to-end testing strategies.

## Test Structure Overview

The MIR runtime follows Rust testing conventions with tests organized across multiple levels:

### 1. Unit Tests (`#[cfg(test)]` modules)

Located within each source file, these test individual functions and methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        // Test implementation
    }
}
```

### 2. Integration Tests (`tests/` directories)

Located in `crates/*/tests/` directories, these test public APIs and module interactions:

- `crates/types/tests/` - Type system integration tests
- `crates/runtime/tests/` - Runtime coordination tests  
- `crates/backend-wasm/tests/` - WASM backend integration tests

### 3. End-to-End Tests

- JavaScript/Node.js NAPI integration tests
- Distributed system coordination tests
- Hot-module-reloading workflow tests

## Test Categories by Component

### Type System Tests (`crates/types/`)

**Unit Tests:**
- Content-addressable hashing
- Type descriptor operations
- Schema compatibility checking
- Migration path computation
- Concurrent access safety

**Integration Tests:**
- Complete type registration workflows
- Schema evolution scenarios
- Migration chain execution
- Runtime type information trait

**Key Test Files:**
- `src/type_registry.rs` - Core registry functionality
- `tests/comprehensive_type_system_tests.rs` - End-to-end type system tests

### Runtime System Tests (`crates/runtime/`)

**Unit Tests:**
- Change analysis algorithms
- State migration functions
- Consensus protocol logic
- HMR coordinator operations
- CRDT implementations

**Integration Tests:**
- Distributed HMR coordination
- Multi-node consensus scenarios
- State preservation during updates
- Error recovery mechanisms

**Key Test Files:**
- `tests/distributed_hmr_integration_tests.rs` - Distributed coordination tests
- Individual module unit tests in `src/` files

### WASM Backend Tests (`crates/backend-wasm/`)

**Unit Tests:**
- Code generation correctness
- NAPI binding structure
- Memory management operations
- FFI bridge functionality

**Integration Tests:**
- Complete WASM compilation pipeline
- Instance lifecycle management
- Hot-reloading with state preservation
- Memory allocation and cleanup

**Key Test Files:**
- `tests/napi_integration.rs` - NAPI binding tests
- `tests/instance_manager_tests.rs` - Instance management tests
- `tests/memory_management_tests.rs` - Memory operation tests
- `tests/ffi_bridge_tests.rs` - FFI functionality tests

### JavaScript/Node.js Integration Tests

**NAPI Integration:**
- Runtime creation and configuration
- Instance management operations
- Memory allocation and access
- FFI function registration and calls
- Hot-reloading workflows
- Statistics and monitoring

**Key Test Files:**
- `examples/comprehensive_napi_test.js` - Complete NAPI test suite

## Running Tests

### Quick Test Run

```bash
# Run all unit tests
cargo test

# Run tests for specific crate
cargo test -p mir-types

# Run with output
cargo test -- --nocapture
```

### Comprehensive Test Suite

```bash
# Run the complete test suite
./scripts/run_comprehensive_tests.sh
```

This script runs:
- All unit tests across all crates
- Integration tests
- Documentation tests
- Code quality checks (clippy)
- Formatting verification
- NAPI tests (if Node.js available)
- Coverage analysis (if tarpaulin available)

### Individual Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only  
cargo test --test '*'

# Documentation tests
cargo test --doc

# Specific test pattern
cargo test test_type_registry

# Run tests with specific features
cargo test --features "distributed,hot-reload"
```

## Test Configuration

### Environment Variables

```bash
# Enable debug logging in tests
export RUST_LOG=debug

# Enable backtraces for test failures
export RUST_BACKTRACE=1

# Set test timeout (for async tests)
export TOKIO_TEST_TIMEOUT=30s
```

### Cargo Configuration

Test-specific settings are configured in `.cargo/config.toml`:

- Debug assertions enabled
- Overflow checks enabled
- Debug info for better error reporting
- Consistent compilation settings

## Writing New Tests

### Unit Test Guidelines

1. **Test Function Naming**: Use descriptive names starting with `test_`
2. **Test Organization**: Group related tests in the same module
3. **Mock Objects**: Create minimal mock implementations for dependencies
4. **Error Cases**: Test both success and failure scenarios
5. **Edge Cases**: Test boundary conditions and invalid inputs

Example:
```rust
#[test]
fn test_type_registry_handles_duplicate_registration() {
    let registry = TypeRegistry::new();
    let hash = ContentHash::new(b"test_type");
    let descriptor = create_test_descriptor(hash);
    
    // First registration should succeed
    let result1 = registry.register_type(descriptor.clone());
    assert_eq!(result1, hash);
    
    // Second registration should return same hash (idempotent)
    let result2 = registry.register_type(descriptor);
    assert_eq!(result2, hash);
    assert_eq!(registry.len(), 1);
}
```

### Integration Test Guidelines

1. **Real Components**: Use actual implementations, not mocks
2. **Realistic Scenarios**: Test workflows that users would perform
3. **Async Testing**: Use `#[tokio::test]` for async operations
4. **Resource Cleanup**: Ensure tests clean up after themselves
5. **Timeout Handling**: Use timeouts for operations that might hang

Example:
```rust
#[tokio::test]
async fn test_distributed_hmr_coordination() {
    let coordinator = DistributedHMRCoordinator::new(node_id).await;
    let proposal = create_test_update_proposal();
    
    let result = timeout(
        Duration::from_secs(10),
        coordinator.coordinate_update(proposal)
    ).await;
    
    assert!(result.is_ok());
    // Verify coordination succeeded
}
```

### NAPI Test Guidelines

1. **Error Handling**: Test both success and error paths
2. **Type Conversion**: Verify JavaScript ↔ Rust type conversions
3. **Memory Safety**: Test memory allocation and cleanup
4. **Concurrent Access**: Test thread safety of NAPI operations
5. **Resource Management**: Verify proper resource cleanup

## Test Coverage Goals

### Minimum Coverage Targets

- **Unit Tests**: 90% line coverage per module
- **Integration Tests**: All public APIs covered
- **Error Paths**: All error conditions tested
- **Edge Cases**: Boundary conditions and invalid inputs

### Coverage Analysis

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir target/coverage

# View report
open target/coverage/tarpaulin-report.html
```

## Continuous Integration

### GitHub Actions Workflow

The test suite is designed to run in CI environments:

1. **Matrix Testing**: Test across multiple Rust versions
2. **Platform Testing**: Test on Linux, macOS, and Windows
3. **Feature Testing**: Test different feature combinations
4. **Performance Testing**: Monitor test execution time
5. **Coverage Reporting**: Upload coverage to codecov

### Local CI Simulation

```bash
# Simulate CI environment
export CI=true
export RUST_BACKTRACE=1

# Run tests as CI would
cargo test --all --verbose
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Debugging Test Failures

### Common Issues and Solutions

1. **Flaky Tests**: Use deterministic test data and proper synchronization
2. **Resource Leaks**: Ensure proper cleanup in test teardown
3. **Timing Issues**: Use appropriate timeouts and synchronization
4. **Platform Differences**: Use conditional compilation for platform-specific tests

### Debug Techniques

```bash
# Run single test with debug output
cargo test test_name -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test test_name

# Run with backtrace
RUST_BACKTRACE=full cargo test test_name

# Run in release mode (for performance issues)
cargo test --release test_name
```

## Performance Testing

### Benchmarking

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn bench_type_registration() {
        let registry = TypeRegistry::new();
        let start = Instant::now();
        
        for i in 0..1000 {
            let data = format!("type_{}", i);
            let descriptor = create_test_descriptor(data.as_bytes());
            registry.register_type(descriptor);
        }
        
        let duration = start.elapsed();
        println!("Registered 1000 types in {:?}", duration);
        assert!(duration.as_millis() < 100); // Performance assertion
    }
}
```

### Memory Usage Testing

```rust
#[test]
fn test_memory_efficiency() {
    let registry = TypeRegistry::new();
    
    // Measure memory before
    let initial_memory = get_memory_usage();
    
    // Perform operations
    for i in 0..10000 {
        let descriptor = create_test_descriptor(&format!("type_{}", i));
        registry.register_type(descriptor);
    }
    
    // Measure memory after
    let final_memory = get_memory_usage();
    let memory_per_type = (final_memory - initial_memory) / 10000;
    
    // Assert reasonable memory usage
    assert!(memory_per_type < 1024); // Less than 1KB per type
}
```

## Test Maintenance

### Regular Maintenance Tasks

1. **Update Test Data**: Keep test data current with schema changes
2. **Review Coverage**: Regularly check and improve test coverage
3. **Performance Monitoring**: Track test execution time trends
4. **Dependency Updates**: Update test dependencies and mocks
5. **Documentation**: Keep test documentation current

### Test Refactoring

- Extract common test utilities to shared modules
- Use test fixtures for complex setup scenarios
- Parameterize tests to reduce duplication
- Group related tests into logical modules

This comprehensive testing strategy ensures the MIR runtime maintains high quality and reliability across all components and use cases.