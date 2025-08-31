# Testing Guide for Viewyard

This document describes the comprehensive test suite for viewyard, a multi-repository workspace management tool.

## Test Architecture

Viewyard uses a multi-layered testing approach:

### 1. Property-Based Tests (`tests/property_tests.rs`)
- **Purpose**: Test invariants across the entire input space using `proptest`
- **Coverage**: Data model validation, serialization/deserialization, configuration parsing
- **Key Features**:
  - Generates thousands of random valid inputs
  - Tests Repository, Viewset, and ViewsetsConfig models
  - Validates YAML serialization roundtrips
  - Ensures data integrity constraints

### 2. Unit Tests (`tests/unit_tests.rs`)
- **Purpose**: Test individual functions and modules in isolation
- **Coverage**: Configuration loading, model methods, error handling
- **Key Features**:
  - Uses temporary directories for filesystem tests
  - Tests both success and failure scenarios
  - Validates error messages and edge cases

### 3. Integration Tests (`tests/integration_tests.rs`)
- **Purpose**: Test complete workflows end-to-end using the CLI
- **Coverage**: Command-line interface, file system operations, user workflows
- **Key Features**:
  - Uses `assert_cmd` for CLI testing
  - Tests with temporary directories and mock configurations
  - Validates stdout/stderr output

### 4. Persona-Based Tests (`tests/persona_tests.rs`)
- **Purpose**: Test specific user scenarios and error handling
- **Coverage**: Two main personas with different needs and skill levels

#### Novice Developer Persona
- Basic git knowledge only
- Makes common mistakes (wrong directories, typos, invalid repo names)
- Needs clear error messages and guidance
- **Tests**: Configuration errors, helpful error messages, command validation

#### Expert Developer Persona  
- Works with multiple organizations and dozens of repositories
- Complex workflows with many viewsets and concurrent views
- Expects efficient operations and advanced features
- **Tests**: Large configurations, performance, complex scenarios

### 5. Simple Integration Tests (`tests/simple_integration.rs`)
- **Purpose**: Basic smoke tests for core functionality
- **Coverage**: Help command, basic CLI validation

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suites
```bash
# Property-based tests
cargo test --test property_tests

# Unit tests  
cargo test --test unit_tests

# Integration tests
cargo test --test integration_tests

# Persona-based tests
cargo test --test persona_tests

# Simple integration tests
cargo test --test simple_integration
```

### Run with Test Runner Script
```bash
./test-runner.sh
```

The test runner provides colored output and runs quality checks alongside tests.

## Test Dependencies

The test suite uses minimal dependencies:
- `proptest` - Property-based testing framework
- `tempfile` - Temporary directory management
- `assert_cmd` - CLI testing utilities
- `predicates` - Assertion predicates for CLI output

## Quality Tools Integration

Tests work alongside Cargo's built-in quality tools:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Run linter with warnings as errors
cargo clippy -- -D warnings

# Run tests
cargo test
```

## CI/CD Integration

The test suite is designed for CI/CD pipelines:

1. **Fast execution** - Most tests complete in under 5 seconds
2. **Deterministic** - No flaky tests or race conditions  
3. **Isolated** - Each test uses its own temporary directory
4. **Comprehensive** - Covers success paths, error cases, and edge conditions

### Example CI Configuration

```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
```

## Test Coverage

The test suite provides comprehensive coverage:

- **Models**: 100% of public API tested with property-based tests
- **Configuration**: All loading scenarios and error cases covered
- **CLI**: All commands and error paths tested
- **User Scenarios**: Both novice and expert developer workflows covered
- **Error Handling**: All error conditions have dedicated tests

## Writing New Tests

### Property-Based Tests
Use proptest for testing invariants:

```rust
proptest! {
    #[test]
    fn my_property_test(input in my_generator()) {
        // Test invariant that should always hold
        prop_assert!(invariant_holds(&input));
    }
}
```

### Integration Tests
Use assert_cmd for CLI testing:

```rust
#[test]
fn test_my_command() {
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("my-command");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

### Unit Tests
Use temporary directories for filesystem tests:

```rust
#[test]
fn test_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Test implementation
}
```

## Performance Considerations

- Property-based tests are limited to reasonable input sizes
- Integration tests use timeouts to prevent hanging
- Large configuration tests verify performance doesn't degrade
- All tests should complete within 30 seconds total

## Debugging Tests

### Verbose Output
```bash
cargo test -- --nocapture
```

### Run Single Test
```bash
cargo test test_name
```

### Debug Property-Based Test Failures
Proptest automatically shrinks failing inputs to minimal examples, making debugging easier.

## Maintenance

- Review test coverage when adding new features
- Update persona tests when CLI behavior changes  
- Keep property-based test generators in sync with data models
- Ensure integration tests reflect actual user workflows
