# Testing Guide

Learn how to run tests, write new tests, and benchmark the Pitchlake Coprocessor.

## Table of Contents
- [Test Structure](#test-structure)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [Benchmarking](#benchmarking)
- [Test Data](#test-data)
- [Debugging Tests](#debugging-tests)

## Test Structure

### Test Organization

The Pitchlake Coprocessor uses Rust's built-in testing framework with tests organized in three ways:

1. **Unit Tests** - Inside `src/` files with `#[cfg(test)]` modules
2. **Integration Tests** - In `tests/` directories within each crate
3. **Method Tests** - Host programs that test full zkVM execution

```
pitchlake-coprocessor/
├── common/
│   └── src/
│       ├── floating_point.rs       # Unit tests for FP calculations
│       ├── fixed_point.rs          # Unit tests for fixed-point math
│       └── tests/
│           └── mock.rs             # Mock data generators
├── methods/
│   └── *-methods/
│       ├── src/lib.rs              # Integration tests
│       └── guest/src/main.rs       # zkVM guest program
└── mains/
    └── */
        └── src/main.rs             # End-to-end method tests
```

### Test Categories

| Category | Location | Purpose | Run Time |
|----------|----------|---------|----------|
| Unit Tests | `src/*.rs` | Test individual functions | < 1 second |
| Integration Tests | `tests/*.rs` | Test crate interfaces | < 5 seconds |
| Method Tests (Dev Mode) | `mains/*/src/main.rs` | Test zkVM execution | 1-2 minutes |
| Method Tests (Production) | `mains/*/src/main.rs` | Test real proofs | 5-10 minutes |

## Running Tests

### Run All Tests

```bash
# Run all tests in the workspace
cargo test --all

# Run tests with output visible
cargo test --all -- --nocapture

# Run tests in release mode (faster)
cargo test --all --release
```

### Run Tests for Specific Crate

```bash
# Test common library
cargo test -p common

# Test TWAP method
cargo test -p twap-error-bound-floating-methods

# Test max return method
cargo test -p max-return-floating-methods

# Test core types
cargo test -p core
```

### Run Specific Test

```bash
# Run single test by name
cargo test test_calculate_twap

# Run tests matching pattern
cargo test twap

# Run tests in specific module
cargo test floating_point::tests
```

### Run Method End-to-End

```bash
# Mock proof (fast, dev mode)
cd mains/mock-proof-composition
RISC0_DEV_MODE=1 cargo run --release

# Individual method
cd mains/twap-error-bound-floating
RISC0_DEV_MODE=1 cargo run --release

# Full proof composition (slow)
cd mains/proof-composition-twap-maxreturn-reserveprice-floating-hashing
RISC0_DEV_MODE=1 cargo run --release
```

## Writing Tests

### Unit Test Example

```rust
// In common/src/floating_point.rs

pub fn calculate_twap(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_twap_simple() {
        let data = vec![10.0, 20.0, 30.0];
        let result = calculate_twap(&data);
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_calculate_twap_empty() {
        let data = vec![];
        let result = calculate_twap(&data);
        assert!(result.is_nan());
    }

    #[test]
    fn test_calculate_twap_with_tolerance() {
        let data = vec![10.0, 20.0, 30.0];
        let result = calculate_twap(&data);
        let expected = 20.0;
        let tolerance = 0.0001;
        assert!((result - expected).abs() < tolerance);
    }
}
```

### Integration Test Example

```rust
// In methods/twap-error-bound-floating-methods/tests/integration_test.rs

use twap_error_bound_floating_methods::*;
use core::TwapErrorBoundInput;

#[test]
fn test_twap_method_with_valid_data() {
    let data: Vec<f64> = (0..2160)
        .map(|i| 15.0 + (i as f64 * 0.01))
        .collect();

    let expected_twap = calculate_expected_twap(&data);

    let input = TwapErrorBoundInput {
        avg_hourly_gas_fee: data,
        twap_tolerance: 1.0,  // 1% tolerance
        twap_result: expected_twap,
    };

    // This would generate a proof in real execution
    // In test mode, it validates the calculation
    let result = validate_twap_calculation(&input);
    assert!(result.is_ok());
}

fn calculate_expected_twap(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}
```

### Method Test Example

```rust
// In mains/twap-error-bound-floating/src/main.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twap_method_execution() {
        // Set up test environment
        std::env::set_var("RISC0_DEV_MODE", "1");

        // Load test data
        let data = load_test_data();

        // Run method
        let result = run_twap_method(&data);

        // Verify output
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.twap_result > 0.0);
    }

    fn load_test_data() -> Vec<f64> {
        // Generate or load test data
        (0..2160).map(|i| 15.0 + (i as f64 * 0.01)).collect()
    }
}
```

### Test Data Generators

```rust
// In common/src/tests/mock.rs

pub fn generate_mock_gas_fees(count: usize, base: f64) -> Vec<f64> {
    (0..count)
        .map(|i| {
            let hour_of_day = i % 24;
            let daily_variation = (hour_of_day as f64 * 0.5).sin() * 3.0;
            base + daily_variation
        })
        .collect()
}

pub fn get_5760_avg_base_fees() -> Vec<f64> {
    generate_mock_gas_fees(5760, 15.0)
}

pub fn get_2160_avg_base_fees() -> Vec<f64> {
    generate_mock_gas_fees(2160, 15.0)
}
```

## Benchmarking

### Measure Execution Time

```bash
# Benchmark individual method
cd mains/twap-error-bound-floating
time RISC0_DEV_MODE=1 cargo run --release

# Benchmark full proof composition
cd mains/proof-composition-twap-maxreturn-reserveprice-floating-hashing
time RISC0_DEV_MODE=1 cargo run --release
```

### Performance Comparison

```bash
# Dev mode (no proving)
time RISC0_DEV_MODE=1 cargo run --release

# Production mode (with proving)
time RISC0_DEV_MODE=0 cargo run --release
```

### Expected Performance

| Operation | Dev Mode | Production Mode |
|-----------|----------|-----------------|
| Mock Proof Composition | ~1 second | N/A |
| Single Sub-Proof | ~30-60 seconds | ~2-3 minutes |
| Full Proof Composition | ~2 minutes | ~6-10 minutes |
| Proof Verification | ~100 ms | ~100 ms |

### Profiling with `cargo flamegraph`

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cd mains/mock-proof-composition
cargo flamegraph --release

# Open generated flamegraph.svg in browser
```

## Test Data

### Using Compressed Data File

```bash
# Extract data.csv from compressed file
gunzip -c data.csv.gz > data.csv

# Copy to method directory
cp data.csv mains/twap-error-bound-floating/
```

### Data Format

```csv
timestamp,gas_price
1708833600,15.234
1708837200,16.789
1708840800,14.567
...
```

- **Rows**: 5760 (240 days × 24 hours)
- **Columns**: timestamp (Unix), gas_price (gwei)

### Mock Data Generators

```rust
use common::tests::mock::*;

// Generate 5760 hours of mock data
let data = get_5760_avg_base_fees();

// Generate 2160 hours (90 days) of mock data
let data = get_2160_avg_base_fees();

// Generate custom mock data
let data = generate_mock_gas_fees(1000, 20.0);
```

## Debugging Tests

### Enable Debug Logging

```bash
# Set log level to debug
export RUST_LOG=debug

# Run tests with logs
cargo test -- --nocapture

# Run specific test with logs
cargo test test_calculate_twap -- --nocapture
```

### Print Values in Tests

```rust
#[test]
fn test_with_debug_output() {
    let data = vec![10.0, 20.0, 30.0];
    let result = calculate_twap(&data);

    // Print to stdout (only visible with --nocapture)
    println!("Input data: {:?}", data);
    println!("TWAP result: {}", result);

    // Use dbg! macro for better output
    dbg!(&data, &result);

    assert_eq!(result, 20.0);
}
```

### Test with Different Tolerances

```rust
#[test]
fn test_twap_with_tight_tolerance() {
    let data = generate_test_data();
    let expected = 15.234;

    // Test with 0.1% tolerance
    let result = validate_twap(data.clone(), expected, 0.1);
    assert!(result.is_ok());

    // Test with 0.01% tolerance (might fail)
    let result = validate_twap(data, expected, 0.01);
    // Check if tolerance is too tight
    if result.is_err() {
        println!("Tolerance too tight: {:?}", result.err());
    }
}
```

### Debug zkVM Execution

```bash
# Enable RISC Zero debug mode
export RISC0_DEBUG=1
export RUST_LOG=risc0=debug

# Run method
cd mains/twap-error-bound-floating
RISC0_DEV_MODE=1 cargo run --release
```

## Common Test Patterns

### Testing with Assertions

```rust
#[test]
fn test_with_assertions() {
    let result = calculate_something();

    // Equality
    assert_eq!(result, expected);

    // Inequality
    assert_ne!(result, wrong_value);

    // Boolean condition
    assert!(result > 0.0);

    // Floating-point comparison
    assert!((result - expected).abs() < 1e-6);
}
```

### Testing Error Cases

```rust
#[test]
#[should_panic(expected = "Data cannot be empty")]
fn test_empty_data_panics() {
    let data = vec![];
    calculate_twap(&data);  // Should panic
}

#[test]
fn test_invalid_input_returns_error() {
    let data = vec![];
    let result = safe_calculate_twap(&data);
    assert!(result.is_err());
}
```

### Testing with Fixtures

```rust
struct TestFixture {
    data: Vec<f64>,
    expected_twap: f64,
    expected_max_return: f64,
}

impl TestFixture {
    fn new() -> Self {
        let data = generate_test_data();
        TestFixture {
            expected_twap: calculate_expected_twap(&data),
            expected_max_return: calculate_expected_max_return(&data),
            data,
        }
    }
}

#[test]
fn test_with_fixture() {
    let fixture = TestFixture::new();
    let result = calculate_twap(&fixture.data);
    assert_eq!(result, fixture.expected_twap);
}
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install RISC Zero
        run: |
          curl -L https://risczero.com/install | bash
          $HOME/.risc0/bin/rzup install

      - name: Run tests
        run: cargo test --all --release
        env:
          RISC0_DEV_MODE: 1
```

## Next Steps

- [Architecture Overview](../architecture/overview.md) - Understand the system design
- [Running Methods](../guides/running-methods.md) - Execute individual methods
- [Integration Guide](../guides/integration-guide.md) - Integrate with your application

## Quick Reference

```bash
# Run all tests
cargo test --all

# Run tests for specific crate
cargo test -p common

# Run specific test
cargo test test_calculate_twap

# Run with output visible
cargo test -- --nocapture

# Run method test
cd mains/mock-proof-composition
RISC0_DEV_MODE=1 cargo run --release

# Benchmark
time RISC0_DEV_MODE=1 cargo run --release

# Debug mode
RUST_LOG=debug cargo test -- --nocapture
```
