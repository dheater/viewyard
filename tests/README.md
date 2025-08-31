# Viewyard Onboarding Test Suite

Comprehensive behavior tests for all Viewyard onboarding functionality, covering all issues discovered during development and testing.

## Test Structure

### 1. Unit Tests (`test_onboarding.py`)
Tests individual components and functions:

- **Prerequisites checking** - Git, Just, Python, PyYAML availability
- **Git configuration detection** - Reading global and context-specific configs
- **Repository discovery** - Local repos, GitHub personal/org repos, private repos
- **Fuzzy search** - Exact match, starts-with, contains, case-insensitive
- **Directory validation** - Empty, existing content, git repos, other files
- **Viewset creation** - Config generation, directory structure
- **Edge cases** - Missing dependencies, API failures, invalid inputs

### 2. Integration Tests (`test_integration.py`)
Tests complete user workflows:

- **Complete onboarding flow** - Two contexts (work/personal)
- **Single context flow** - Work-only setup
- **Manual entry flow** - Adding repos manually
- **Mixed mode flow** - Search → Manual → Search transitions
- **Error handling** - Existing directories, API failures
- **Regression tests** - Specific bugs found during development

### 3. Manual Tests
Interactive tests requiring user verification:

- Email pre-population from context configs
- Multiple viewset creation experience
- Search/manual mode transitions
- Private repository discovery
- Directory structure validation
- Search consistency across queries

## Issues Covered

All tests cover the specific issues discovered during development:

### 🔧 **Fixed Issues**
- ✅ **Email Detection**: Emails not detected from `~/.gitconfig-work` and `~/.gitconfig-personal`
- ✅ **Smart Git Context Detection**: Now detects existing configs like `~/.gitconfig-imprivata`, `~/.gitconfig-dheater`
- ✅ **Context Name Preservation**: Uses existing context names instead of creating duplicate `work`/`personal` configs
- ✅ **Email Pattern Mapping**: Correctly maps work contexts (imprivata, daniel.heater) vs personal contexts (dheater, @pm.me)
- ✅ **SSH Authentication**: Fixed SSH host alias usage (`git@github.com-dheater` vs `git@github.com`)
- ✅ **Private Repos**: `audit` repo not found due to private status
- ✅ **Search Flow**: Manual entry didn't return to search mode
- ✅ **Multiple Viewsets**: Only created one viewset instead of work + personal
- ✅ **Directory Structure**: Used `src-work` instead of clean `work` paths
- ✅ **Search Consistency**: Results inconsistent after multiple queries
- ✅ **Directory Creation**: Viewset directories not created during onboarding
- ✅ **GitHub Account-Based Naming**: Viewset names default to actual GitHub account names

### 🧪 **Edge Cases Tested**
- ✅ **Empty Directories**: Safe to proceed
- ✅ **Existing Content**: Warn and offer options
- ✅ **Git Repositories**: Detect and offer import
- ✅ **Existing Git Configs**: Detect and preserve existing `~/.gitconfig-*` files
- ✅ **Context Name Conflicts**: Handle when existing contexts don't match work/personal pattern
- ✅ **SSH Key Conflicts**: Detect and correct SSH host alias usage
- ✅ **Multiple GitHub Accounts**: Handle authentication switching between accounts
- ✅ **GitHub CLI Missing**: Graceful fallback
- ✅ **API Failures**: Handle network/auth issues
- ✅ **Invalid URLs**: Skip malformed repository URLs
- ✅ **Interrupted Flow**: Handle Ctrl+C gracefully
- ✅ **Symlinked Dotfiles**: Handle symlinked git config files correctly

## Running Tests

### Quick Start
```bash
# Run all automated tests
python tests/run_tests.py

# Run specific test suites
python -m pytest tests/test_onboarding.py -v
python -m pytest tests/test_integration.py -v
```

### Prerequisites
```bash
pip install pytest pyyaml
```

### Test Categories

#### 1. Automated Tests (CI-ready)
```bash
# Unit tests - individual components
pytest tests/test_onboarding.py::TestPrerequisites -v
pytest tests/test_onboarding.py::TestGitConfiguration -v
pytest tests/test_onboarding.py::TestRepositoryDiscovery -v
pytest tests/test_onboarding.py::TestFuzzySearch -v
pytest tests/test_onboarding.py::TestDirectoryValidation -v

# Integration tests - complete workflows
pytest tests/test_integration.py::TestCompleteOnboardingFlow -v
pytest tests/test_integration.py::TestErrorHandlingAndEdgeCases -v
pytest tests/test_integration.py::TestRegressionTests -v
```

#### 2. Manual Tests (User verification)
```bash
# Run actual onboarding to verify:
python scripts/onboard.py

# Check these behaviors:
# - Email pre-population from context configs
# - Multiple viewset creation (work + personal)
# - Search → manual → search transitions
# - Private repo discovery (audit repo)
# - Clean directory structure (~/src/work/ not ~/src/src-work/)
```

## Recent Improvements Tested

### **Smart Git Context Detection** 🧠
The test suite now covers the enhanced git configuration detection that:
- **Detects existing context configs** like `~/.gitconfig-imprivata`, `~/.gitconfig-dheater`
- **Preserves existing context names** instead of creating generic `work`/`personal` configs
- **Maps contexts by email patterns** (work: imprivata, daniel.heater; personal: dheater, @pm.me)
- **Prevents duplicate config creation** when existing configs are found

### **SSH Authentication Testing** 🔐
New tests verify the SSH host alias fixes:
- **SSH host alias detection** (`github.com-dheater`, `github.com-imprivata`)
- **Remote URL correction** (from `git@github.com:` to `git@github.com-dheater:`)
- **Multi-account authentication** handling
- **SSH key conflict resolution**

### **GitHub Account-Based Naming** 🏷️
Tests for the improved viewset naming logic:
- **Account name detection** from repository sources
- **Smart default suggestions** based on actual GitHub accounts
- **Context-aware naming** (work gets `daniel-heater-imprivata`, personal gets `dheater`)
- **Fallback naming** when accounts can't be detected

## Test Coverage

### Components Tested
- [x] Prerequisites checking
- [x] Git configuration detection and parsing
- [x] Smart git context detection (existing `~/.gitconfig-*` files)
- [x] Context name preservation and mapping
- [x] Email pattern recognition for work/personal contexts
- [x] SSH authentication and host alias handling
- [x] Repository discovery (local + GitHub)
- [x] Private repository handling
- [x] GitHub account-based naming suggestions
- [x] Fuzzy search functionality
- [x] Directory validation and analysis
- [x] Viewset creation and configuration
- [x] User input handling and validation
- [x] Error handling and recovery
- [x] Integration between all components

### User Workflows Tested
- [x] First-time onboarding (clean slate)
- [x] Onboarding with existing git configs
- [x] Single context setup (work only)
- [x] Dual context setup (work + personal)
- [x] Repository search and selection
- [x] Manual repository entry
- [x] Mixed search/manual workflows
- [x] Existing directory handling
- [x] Error recovery scenarios

### Regression Tests
- [x] Private repository discovery (audit repo issue)
- [x] Email detection from context-specific configs
- [x] Search consistency across multiple queries
- [x] Directory structure using clean paths
- [x] Manual mode returning to search
- [x] Multiple viewset creation

## Test Data

Tests use realistic mock data based on actual usage:

```python
# Example repositories used in tests
mock_repos = [
    {"name": "audit", "url": "https://github.com/imprivata-pas/audit", "source": "GitHub (imprivata-pas) [private]"},
    {"name": "librssconnect", "url": "https://github.com/imprivata-pas/librssconnect", "source": "GitHub (imprivata-pas) [private]"},
    {"name": "universal-connection-manager", "url": "https://github.com/imprivata-pas/universal-connection-manager", "source": "GitHub (imprivata-pas)"}
]

# Example git configurations with realistic context names
imprivata_config = """[user]
    name = "Daniel Heater"
    email = "daniel.heater@imprivata.com"
    signingkey = ~/.ssh/id_ed25519.pub
"""

dheater_config = """[user]
    name = "Daniel Heater"
    email = "dheater@pm.me"
    signingkey = ~/.ssh/id-dheater.pub
"""

# SSH configuration for multiple GitHub accounts
ssh_config = """
Host github.com-dheater
    HostName github.com
    User git
    IdentityFile ~/.ssh/id-dheater

Host github.com-imprivata
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519
"""
```

## Continuous Integration

Tests are designed to run in CI environments:

```yaml
# Example GitHub Actions workflow
- name: Run Viewyard Tests
  run: |
    pip install pytest pyyaml
    python tests/run_tests.py
```

## Contributing

When adding new functionality:

1. **Add unit tests** for individual components
2. **Add integration tests** for user workflows
3. **Update manual tests** for UI/UX changes
4. **Add regression tests** for any bugs found
5. **Update this README** with new test coverage

### Test Naming Convention
- `test_<component>_<scenario>` for unit tests
- `test_<workflow>_flow` for integration tests
- `test_<issue>_regression` for regression tests

## Debugging Tests

### Common Issues
```bash
# Import errors
export PYTHONPATH="${PYTHONPATH}:$(pwd)/scripts"

# Missing dependencies
pip install pytest pyyaml

# Mock failures
# Check that mocks match actual function signatures
```

### Verbose Output
```bash
# See detailed test output
pytest tests/ -v -s

# See print statements
pytest tests/ -v -s --capture=no
```

This test suite ensures that all discovered issues are covered and prevents regressions as the codebase evolves.
