# Contributing to Viewyard

Thank you for your interest in contributing to Viewyard! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Development Setup

1. **Clone the repository**
```bash
git clone https://github.com/dheater/viewyard.git
cd viewyard
```

2. **Install Rust** (if not already installed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

3. **Install dependencies**
```bash
# GitHub CLI (required for testing)
brew install gh  # macOS
# or follow instructions at https://cli.github.com/

# Authenticate with GitHub CLI
gh auth login
```

4. **Build and test**
```bash
cargo build
cargo test
cargo run -- --help
```

## 🏗️ Project Structure

```
viewyard/
├── src/
│   ├── main.rs              # Main entry point and CLI handling
│   ├── commands/
│   │   └── workspace.rs     # Workspace commands (status, commit-all, etc.)
│   ├── git.rs              # Git operations wrapper
│   ├── github.rs           # GitHub API integration
│   ├── interactive.rs      # Interactive repository selection
│   ├── models.rs           # Data structures
│   ├── search.rs           # Repository search and filtering
│   └── ui.rs               # User interface utilities
├── tests/                  # Integration tests
├── Cargo.toml             # Rust dependencies and metadata
└── README.md              # Project documentation
```

## 🧪 Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests
```

### Test Categories
- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test command-line interface and workflows
- **Real workflow tests**: Test actual git operations (requires setup)

### Writing Tests
- Add unit tests in the same file as the code being tested
- Add integration tests in the `tests/` directory
- Use descriptive test names that explain what is being tested
- Include both positive and negative test cases

## 🐛 Bug Reports

### Before Reporting
1. **Search existing issues** to avoid duplicates
2. **Test with latest version** to ensure bug still exists
3. **Gather information** about your environment

### Bug Report Template
```markdown
**Description**
A clear description of the bug.

**Steps to Reproduce**
1. Run command: `viewyard ...`
2. Expected behavior: ...
3. Actual behavior: ...

**Environment**
- OS: [e.g., macOS 12.0, Ubuntu 20.04]
- Viewyard version: [e.g., 0.2.0]
- GitHub CLI version: [e.g., 2.0.0]
- Git version: [e.g., 2.30.0]

**Additional Context**
Any additional information that might be helpful.
```

## 💡 Feature Requests

### Before Requesting
1. **Check existing issues** for similar requests
2. **Consider the scope** - does it fit Viewyard's goals?
3. **Think about implementation** - how would it work?

### Feature Request Template
```markdown
**Problem Statement**
What problem does this feature solve?

**Proposed Solution**
How should this feature work?

**Alternatives Considered**
What other approaches did you consider?

**Additional Context**
Any additional information or examples.
```

### Not Currently Seeking
- **Major architectural changes**: Core design is stable
- **New commands**: Focus is on simplicity and core functionality
- **Complex features**: Prefer simple, focused solutions

## 🔧 Development Guidelines

### Commit Messages
- **Use conventional commits**: `feat:`, `fix:`, `docs:`, `test:`, etc.
- **Be descriptive**: Explain what and why, not just what
- **Reference issues**: Include issue numbers when relevant

Example:
```
feat: add rebase conflict detection

Detect when git rebase fails due to conflicts and provide
specific recovery instructions to help users resolve issues.

Fixes #123
```

### Pull Request Process

I am not accepting PRs at this time. Please create an issue. Providing a patch in the issue is welcome.

## 🎯 Development Priorities

### Current Focus Areas
1. **Data Safety**: Ensuring operations are atomic and safe
2. **Error Handling**: Providing clear, actionable error messages
3. **User Experience**: Making the tool intuitive and helpful
4. **Performance**: Keeping operations fast and efficient

## 🔒 Security Considerations

### Security-First Development
- **Validate inputs**: Always validate user inputs and external data
- **Handle credentials safely**: Never log or expose credentials
- **Use safe defaults**: Choose secure options by default
- **Minimize permissions**: Request only necessary permissions

## 🤝 Community Guidelines

### Code of Conduct
- **Be respectful**: Treat all contributors with respect
- **Be inclusive**: Welcome contributors from all backgrounds
- **Be constructive**: Provide helpful feedback and suggestions
- **Be patient**: Remember that everyone is learning
