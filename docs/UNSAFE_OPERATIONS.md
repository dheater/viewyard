# Unsafe Operations - Security Constraints

## Critical Security Rules

### **NEVER Modify Global Git Configuration**

**Rule**: Viewyard MUST NEVER modify global git configuration under any circumstances.

**Rationale**: 
- Viewyard operates on multiple repositories from different accounts/organizations
- Global git config modifications would pollute the user's environment
- Could cause commits to be attributed to wrong accounts
- Violates principle of least privilege and environment isolation

**Enforcement**:
- ✅ `set_git_config()` function hardcoded to use `--local` flag only
- ✅ `GitConfigScope` enum prevents global modifications via type system
- ✅ Test `test_global_config_never_modified()` verifies this constraint
- ✅ Code review must reject any `--global` flag usage

**Allowed**:
- ✅ Reading global config via `get_global_git_config()` (read-only)
- ✅ Setting repository-local config via `set_git_config()`

**Forbidden**:
- ❌ Any `git config --global` modifications
- ❌ Modifying `~/.gitconfig` file directly
- ❌ Setting global git configuration in tests

### **Test Safety Requirements**

**Rule**: Tests MUST NOT modify global system state.

**Specific Constraints**:
- ❌ No global git config modifications
- ❌ No modifications to `~/.gitconfig`, `~/.ssh/config`, etc.
- ❌ No installation of global git hooks
- ✅ Use temporary directories and repositories only
- ✅ Clean up all test artifacts

**Example Violations**:
```rust
// FORBIDDEN - modifies global git config
std::process::Command::new("git")
    .args(["config", "--global", "user.signingkey", "test-key"])
    .output()?;

// FORBIDDEN - modifies global SSH config  
fs::write("~/.ssh/config", "test config")?;
```

**Safe Alternatives**:
```rust
// SAFE - repository-local config only
set_git_config("user.signingkey", "test-key", repo_path)?;

// SAFE - temporary test repositories
let temp_repo = TempDir::new()?;
```

### **File System Safety**

**Rule**: Only modify files within designated workspace directories.

**Allowed**:
- ✅ Creating/modifying files in current working directory
- ✅ Repository-specific `.git/config` files
- ✅ Temporary directories created by tests

**Forbidden**:
- ❌ Modifying files in user's home directory
- ❌ Modifying system-wide configuration files
- ❌ Installing global git hooks or templates

### **Network Safety**

**Rule**: Only make necessary API calls with user consent.

**Allowed**:
- ✅ GitHub API calls via authenticated `gh` CLI
- ✅ Git operations to user's repositories

**Forbidden**:
- ❌ Telemetry or analytics data collection
- ❌ Unauthorized API calls
- ❌ Data transmission to third-party services

## Code Review Checklist

When reviewing code, check for:

- [ ] No `--global` flags in git commands
- [ ] No modifications to files outside workspace
- [ ] Tests use temporary directories only
- [ ] No global system state modifications
- [ ] Proper error handling for safety violations

## Enforcement Mechanisms

1. **Type System**: `GitConfigScope` enum prevents global modifications
2. **Function Design**: `set_git_config()` hardcoded to use `--local`
3. **Testing**: `test_global_config_never_modified()` verifies constraints
4. **Documentation**: This file and inline code comments
5. **Code Review**: Manual verification of safety constraints

## Incident Response

If unsafe operations are discovered:

1. **Immediate**: Remove or disable the unsafe code
2. **Assessment**: Determine scope of potential impact
3. **Notification**: Inform users if their environment may be affected
4. **Prevention**: Add tests and constraints to prevent recurrence
5. **Review**: Audit related code for similar issues

## Contact

For security concerns or questions about unsafe operations:
- Create an issue with `security` label
- Review this document before implementing new features
- When in doubt, choose the more restrictive/safer approach
