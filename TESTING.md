# Testing Guide for Viewyard

This document describes the test suite for viewyard, a multi-repository workspace management tool.

## Test Architecture

Viewyard uses a focused testing approach with real integration tests:

### 1. Unit Tests (`tests/unit_tests.rs`)
- **Purpose**: Test individual functions and modules in isolation
- **Coverage**: Repository model, search functionality, data structures
- **Key Features**:
  - Repository creation and validation
  - Fuzzy search functionality
  - Repository grouping and filtering

### 2. Integration Tests (`tests/integration_tests.rs`)
- **Purpose**: Test CLI interface and basic workflows
- **Coverage**: Command-line interface, error handling, help system
- **Key Features**:
  - CLI testing with temporary directories
  - Stdout/stderr validation
  - Error condition testing

### 3. Real Workflow Tests (`tests/real_workflow_tests.rs`)
- **Purpose**: Test actual user workflows and error conditions
- **Coverage**: GitHub CLI integration, viewset/view creation, workspace commands
- **Key Features**:
  - Behavior when GitHub CLI is unavailable
  - Workspace commands outside of views
  - Context detection (viewset vs view vs outside)
  - Help system functionality

### 4. Simple Integration Tests (`tests/simple_integration.rs`)
- **Purpose**: Basic smoke tests for core functionality
- **Coverage**: Help command, basic CLI validation

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suites
```bash
# Unit tests
cargo test --test unit_tests

# Integration tests  
cargo test --test integration_tests

# Real workflow tests
cargo test --test real_workflow_tests

# Simple integration tests
cargo test --test simple_integration
```

## Test Principles

1. **Real workflows** - Test actual user scenarios
2. **Deterministic** - No flaky tests or race conditions
3. **Isolated** - Each test uses its own temporary directory

## Debugging Tests

### Run with Output
```bash
cargo test -- --nocapture
```

### Run Single Test
```bash
cargo test test_name
```

## Maintenance

- Review test coverage when adding new features
- Ensure integration tests reflect actual user workflows
- Keep tests fast and deterministic

## Constraints for writing tests

 CRITICAL: Do not commit or push anything from tests to upstream Github repos.
- Only use disposible local repos.
- Change git origins to point to these local repose for testing.
- Do not make changes to the global git configuration.
