# Auggie Tool Limitations and Workarounds


## View Tool Workspace Restriction

**Issue**: The `view` tool only works within the current workspace directory and cannot access files outside the workspace, even with proper absolute paths and file permissions.

**Reported by**: Carl (custom Auggie personality)  
**Confirmed by**: Rob Kitaoka (Augment team)  
**Date**: August 2025

### Details

- **Expected behavior**: `view` tool should work with any readable absolute path
- **Actual behavior**: Only works within workspace directory
- **Example failure**:
  - File exists: `/Users/dheater/.auggie-memory/personal/context.md` (confirmed with `ls -la`)
  - `view` tool returns: "File not found"

### Official Response from Rob Kitaoka

> "This is intended behavior as the agent will default to what is in the project workspace. You can add additional repos/folders to the context so Augment will be able to see these natively. The agent can use the terminal to access files outside of the workspace but it's easier for the agent to recognize project-related files in the workspace."

### Workarounds

1. **Use `cat` via `launch-process`**: 
   ```bash
   cat /path/to/file/outside/workspace
   ```

2. **Add external directories to workspace context**: Configure Augment to include additional folders in the project context

3. **Copy files to workspace**: Move or copy external files into the current workspace directory

### Impact

This limitation breaks workflows that need to access:
- Configuration files stored outside project directories
- Context files (like `~/.auggie-memory/personal/context.md`)
- System-wide configuration files
- Files in other repositories or directories

### Best Practices

- Store project-related context files within the workspace when possible
- Use `launch-process` with `cat` for accessing external files
- Consider adding frequently accessed external directories to the workspace context
- Document external file dependencies clearly in project documentation
