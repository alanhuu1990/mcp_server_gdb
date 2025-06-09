# Development Workflow

This document outlines the development workflow for the MCP Server GDB project.

## Branch Structure

### Main Branch (`main`)
- **Purpose**: Production-ready, stable releases
- **Protection**: Protected branch, requires PR reviews
- **Deployment**: Tagged releases are built from this branch
- **Status**: Currently at v0.3.0 with full JSON Schema compliance

### Develop Branch (`develop`)
- **Purpose**: Integration branch for new features and fixes
- **Usage**: Active development, feature integration, testing
- **Merging**: Features merge into develop, develop merges into main for releases
- **Status**: Currently synced with main (v0.3.0)

## Repository Setup

### Remote Configuration
```bash
# Your fork (origin)
origin: https://github.com/alanhuu1990/mcp_server_gdb.git

# Original repository (upstream)  
upstream: https://github.com/pansila/mcp_server_gdb.git
```

### Branch Tracking
- `main` tracks `origin/main`
- `develop` tracks `origin/develop`

## Development Workflow

### 1. Starting New Work
```bash
# Switch to develop branch
git checkout develop

# Pull latest changes
git pull origin develop

# Create feature branch
git checkout -b feature/your-feature-name
```

### 2. Feature Development
```bash
# Make your changes
# ... edit files ...

# Stage and commit changes
git add .
git commit -m "feat: description of your feature"

# Push feature branch
git push origin feature/your-feature-name
```

### 3. Integration
```bash
# Switch back to develop
git checkout develop

# Merge feature (or create PR)
git merge feature/your-feature-name

# Push updated develop
git push origin develop

# Clean up feature branch
git branch -d feature/your-feature-name
git push origin --delete feature/your-feature-name
```

### 4. Release Process
```bash
# When ready for release, merge develop to main
git checkout main
git pull origin main
git merge develop

# Tag the release
git tag -a v0.4.0 -m "Release v0.4.0: Description"

# Push main and tags
git push origin main
git push origin --tags
```

## Commit Message Convention

Follow conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples
```bash
git commit -m "feat: add new debugging tool for memory inspection"
git commit -m "fix: resolve JSON schema validation error in breakpoint tool"
git commit -m "docs: update setup instructions for Augment AI"
git commit -m "refactor: improve error handling in GDB manager"
```

## Syncing with Upstream

Periodically sync with the original repository:

```bash
# Fetch upstream changes
git fetch upstream

# Merge upstream main into your main
git checkout main
git merge upstream/main
git push origin main

# Update develop with main changes
git checkout develop
git merge main
git push origin develop
```

## Current Project Status

### v0.3.0 Achievements ✅
- JSON Schema validation issues completely resolved
- All 16 debugging tools working with compliant schemas
- Custom integer types eliminate format specifier problems
- Comprehensive documentation and Augment AI configuration
- Production-ready server with full MCP 2024-11-05 compliance

### Next Development Priorities
1. **Enhanced STM32 Support**: Add more ARM Cortex-M specific debugging features
2. **Performance Optimization**: Improve GDB communication efficiency
3. **Error Handling**: Enhanced error messages and recovery mechanisms
4. **Testing**: Comprehensive test suite for all debugging scenarios
5. **Documentation**: Video tutorials and advanced usage guides

## Development Environment

### Prerequisites
- Rust 1.87.0 or later
- GDB with MI interface support
- STM32 development tools (optional, for testing)

### Building
```bash
# Debug build
cargo build

# Release build  
cargo build --release

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

### Testing
```bash
# Run MCP protocol tests
cargo test mcp

# Test with real GDB session
./target/release/mcp-server-gdb --log-level debug

# Validate JSON Schema
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}' | ./target/release/mcp-server-gdb
```

## Contributing Guidelines

1. **Always work on feature branches** - Never commit directly to main or develop
2. **Write descriptive commit messages** - Follow conventional commits format
3. **Test your changes** - Ensure all tools work and schemas validate
4. **Update documentation** - Keep README and docs in sync with changes
5. **Follow Rust best practices** - Use clippy and rustfmt

## Release Checklist

Before creating a new release:

- [ ] All tests pass
- [ ] JSON Schema validation confirmed
- [ ] Documentation updated
- [ ] CHANGELOG.md updated with new version
- [ ] Version bumped in Cargo.toml
- [ ] Configuration files tested with Augment AI
- [ ] Binary builds successfully in release mode

---

## Quick Reference

```bash
# Current branch status
git status
git branch -a

# Switch to develop for new work
git checkout develop

# Create and push feature branch
git checkout -b feature/my-feature
git push origin feature/my-feature

# Build and test
cargo build --release
./target/release/mcp-server-gdb --help
```

Happy coding! 🚀
