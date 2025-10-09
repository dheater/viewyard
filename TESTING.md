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

### 4. Git Tests (`tests/git_tests.rs`)
- **Purpose**: Test git operations and configuration management
- **Coverage**: Git config operations, branch detection, SSH host aliases
- **Key Features**:
  - Git configuration safety (never modifies global config)
  - Default branch detection across different git setups
  - Repository validation and account extraction

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

# Git tests
cargo test --test git_tests
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
