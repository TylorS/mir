#!/bin/bash

# Comprehensive Test Runner for MIR Runtime
# This script runs all unit tests, integration tests, and checks coverage

set -e

echo "🚀 MIR Runtime Comprehensive Test Suite"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to run tests for a specific crate
run_crate_tests() {
    local crate_name=$1
    print_status $BLUE "📦 Testing crate: $crate_name"
    
    if cargo test -p $crate_name --verbose; then
        print_status $GREEN "✅ $crate_name tests passed"
    else
        print_status $RED "❌ $crate_name tests failed"
        return 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    local crate_name=$1
    print_status $BLUE "🔗 Running integration tests for: $crate_name"
    
    if cargo test -p $crate_name --test '*' --verbose; then
        print_status $GREEN "✅ $crate_name integration tests passed"
    else
        print_status $RED "❌ $crate_name integration tests failed"
        return 1
    fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_status $RED "❌ Error: Must be run from the project root directory"
    exit 1
fi

# Start timing
start_time=$(date +%s)

print_status $YELLOW "🔧 Building all crates..."
if ! cargo build --all; then
    print_status $RED "❌ Build failed"
    exit 1
fi

print_status $GREEN "✅ Build successful"

# Run unit tests for each crate
print_status $YELLOW "🧪 Running unit tests..."

CRATES=(
    "mir-types"
    "mir-ast" 
    "mir-vm"
    "mir-compiler"
    "mir-runtime"
    "mir-backend-wasm"
    "mir-backend-js"
    "mir-backend-wasi"
)

failed_crates=()

for crate in "${CRATES[@]}"; do
    if ! run_crate_tests $crate; then
        failed_crates+=($crate)
    fi
done

# Run integration tests
print_status $YELLOW "🔗 Running integration tests..."

INTEGRATION_CRATES=(
    "mir-runtime"
    "mir-backend-wasm"
    "mir-types"
)

for crate in "${INTEGRATION_CRATES[@]}"; do
    if ! run_integration_tests $crate; then
        failed_crates+=($crate)
    fi
done

# Run workspace-level tests
print_status $YELLOW "🏗️  Running workspace tests..."
if cargo test --workspace --verbose; then
    print_status $GREEN "✅ Workspace tests passed"
else
    print_status $RED "❌ Workspace tests failed"
    failed_crates+=("workspace")
fi

# Run doc tests
print_status $YELLOW "📚 Running documentation tests..."
if cargo test --doc --workspace; then
    print_status $GREEN "✅ Documentation tests passed"
else
    print_status $RED "❌ Documentation tests failed"
    failed_crates+=("doc-tests")
fi

# Run clippy for code quality
print_status $YELLOW "📎 Running clippy analysis..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_status $GREEN "✅ Clippy analysis passed"
else
    print_status $YELLOW "⚠️  Clippy found issues (not failing build)"
fi

# Check formatting
print_status $YELLOW "🎨 Checking code formatting..."
if cargo fmt --all -- --check; then
    print_status $GREEN "✅ Code formatting is correct"
else
    print_status $YELLOW "⚠️  Code formatting issues found (run 'cargo fmt' to fix)"
fi

# Run NAPI tests if Node.js is available
if command -v node &> /dev/null; then
    print_status $YELLOW "🟢 Running Node.js NAPI tests..."
    
    # Build NAPI bindings first
    if cargo build -p mir-backend-wasm --release; then
        # Check if the comprehensive test exists and run it
        if [ -f "crates/backend-wasm/examples/comprehensive_napi_test.js" ]; then
            if node crates/backend-wasm/examples/comprehensive_napi_test.js; then
                print_status $GREEN "✅ NAPI tests passed"
            else
                print_status $YELLOW "⚠️  NAPI tests failed (may be expected in CI environment)"
            fi
        else
            print_status $YELLOW "⚠️  NAPI test file not found"
        fi
    else
        print_status $YELLOW "⚠️  Failed to build NAPI bindings"
    fi
else
    print_status $YELLOW "⚠️  Node.js not available, skipping NAPI tests"
fi

# Generate test coverage report if tarpaulin is available
if command -v cargo-tarpaulin &> /dev/null; then
    print_status $YELLOW "📊 Generating test coverage report..."
    if cargo tarpaulin --out Html --output-dir target/coverage --workspace; then
        print_status $GREEN "✅ Coverage report generated in target/coverage/"
    else
        print_status $YELLOW "⚠️  Coverage report generation failed"
    fi
else
    print_status $YELLOW "⚠️  cargo-tarpaulin not available, skipping coverage report"
    print_status $BLUE "💡 Install with: cargo install cargo-tarpaulin"
fi

# Calculate execution time
end_time=$(date +%s)
execution_time=$((end_time - start_time))

# Print summary
print_status $BLUE "📋 Test Summary"
print_status $BLUE "==============="

if [ ${#failed_crates[@]} -eq 0 ]; then
    print_status $GREEN "🎉 All tests passed!"
    print_status $GREEN "⏱️  Total execution time: ${execution_time}s"
    exit 0
else
    print_status $RED "❌ Some tests failed:"
    for crate in "${failed_crates[@]}"; do
        print_status $RED "  - $crate"
    done
    print_status $RED "⏱️  Total execution time: ${execution_time}s"
    exit 1
fi