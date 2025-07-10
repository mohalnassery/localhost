# Contributing to Localhost HTTP Server

Thank you for your interest in contributing to the Localhost HTTP Server! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Process](#contributing-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Issue Reporting](#issue-reporting)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow:

- **Be respectful** - Treat all community members with respect and kindness
- **Be inclusive** - Welcome newcomers and help them get started
- **Be constructive** - Provide helpful feedback and suggestions
- **Be patient** - Remember that everyone has different experience levels
- **Be collaborative** - Work together to improve the project

## Getting Started

### Prerequisites

- **Rust** 1.70 or later
- **Git** for version control
- **Linux** system for development and testing
- **Python 3** for CGI examples and tests

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/localhost.git
   cd localhost
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/mohalnassery/localhost.git
   ```

## Development Setup

### Build the Project

```bash
# Debug build for development
cargo build

# Release build for testing
cargo build --release
```

### Run Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# All tests with test runner
./tests/run_tests.sh
```

### Development Server

```bash
# Start development server
./target/debug/localhost-server config/test.conf

# Or with release build
./target/release/localhost-server config/test.conf
```

## Contributing Process

### 1. Choose an Issue

- Look for issues labeled `good first issue` for beginners
- Check existing issues before creating new ones
- Comment on issues you'd like to work on

### 2. Create a Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
```

### 3. Make Changes

- Follow the coding standards below
- Write tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 4. Test Your Changes

```bash
# Run all tests
./tests/run_tests.sh

# Run specific test suites
./tests/run_tests.sh unit
./tests/run_tests.sh integration
./tests/run_tests.sh stress

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

### 5. Submit Pull Request

- Push your branch to your fork
- Create a pull request on GitHub
- Fill out the pull request template
- Respond to review feedback

## Coding Standards

### Rust Style

Follow the official Rust style guidelines:

```bash
# Format code
cargo fmt

# Check for common issues
cargo clippy
```

### Code Organization

- **Modules**: Organize code into logical modules
- **Functions**: Keep functions focused and small
- **Error Handling**: Use `Result` types for error handling
- **Documentation**: Document public APIs with `///` comments
- **Tests**: Place unit tests in the same file with `#[cfg(test)]`

### Naming Conventions

- **Variables**: `snake_case`
- **Functions**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Example Code Style

```rust
/// Represents an HTTP request with headers and body
pub struct HttpRequest {
    pub method: HttpMethod,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Creates a new HTTP request
    pub fn new() -> Self {
        Self {
            method: HttpMethod::GET,
            uri: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
    
    /// Gets a header value by name (case-insensitive)
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_request() {
        let request = HttpRequest::new();
        assert_eq!(request.method, HttpMethod::GET);
        assert!(request.uri.is_empty());
    }
}
```

## Testing Guidelines

### Test Categories

1. **Unit Tests** - Test individual functions and methods
2. **Integration Tests** - Test component interactions
3. **End-to-End Tests** - Test complete request/response cycles
4. **Performance Tests** - Test performance characteristics
5. **Security Tests** - Test security features

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
    
    #[test]
    fn test_error_condition() {
        let result = function_that_should_fail();
        assert!(result.is_err());
    }
}
```

### Test Coverage

- Aim for high test coverage on new code
- Test both success and error paths
- Include edge cases and boundary conditions
- Test concurrent scenarios where applicable

## Documentation

### Code Documentation

- Document all public APIs with `///` comments
- Include examples in documentation
- Explain complex algorithms and data structures
- Document error conditions and return values

### User Documentation

- Update README.md for user-facing changes
- Update configuration documentation
- Add examples for new features
- Update deployment guides if needed

### Example Documentation

```rust
/// Parses an HTTP request from raw bytes
/// 
/// # Arguments
/// 
/// * `data` - Raw HTTP request bytes
/// 
/// # Returns
/// 
/// * `Ok(Some(HttpRequest))` - Successfully parsed complete request
/// * `Ok(None)` - Incomplete request, need more data
/// * `Err(HttpError)` - Parse error
/// 
/// # Examples
/// 
/// ```
/// let data = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
/// let request = parse_request(data)?;
/// assert_eq!(request.method, HttpMethod::GET);
/// ```
pub fn parse_request(data: &[u8]) -> Result<Option<HttpRequest>, HttpError> {
    // Implementation
}
```

## Submitting Changes

### Pull Request Guidelines

1. **Title**: Clear, descriptive title
2. **Description**: Explain what changes were made and why
3. **Testing**: Describe how the changes were tested
4. **Documentation**: Note any documentation updates
5. **Breaking Changes**: Highlight any breaking changes

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added for new functionality
```

### Review Process

1. **Automated Checks** - CI/CD pipeline runs tests
2. **Code Review** - Maintainers review code quality
3. **Testing** - Reviewers test functionality
4. **Approval** - Changes approved by maintainers
5. **Merge** - Changes merged to main branch

## Issue Reporting

### Bug Reports

Include the following information:

- **Environment**: OS, Rust version, server version
- **Configuration**: Relevant configuration settings
- **Steps to Reproduce**: Clear reproduction steps
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happens
- **Logs**: Relevant log output or error messages

### Feature Requests

Include the following information:

- **Use Case**: Why is this feature needed?
- **Proposed Solution**: How should it work?
- **Alternatives**: Other solutions considered
- **Implementation**: Any implementation ideas

### Security Issues

For security vulnerabilities:

1. **Do not** create public issues
2. Email security issues to: security@example.com
3. Include detailed reproduction steps
4. Allow time for fix before disclosure

## Development Workflow

### Branch Naming

- `feature/description` - New features
- `bugfix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring

### Commit Messages

Follow conventional commit format:

```
type(scope): description

Longer description if needed

Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release tag
4. Build and test release
5. Publish release notes

## Getting Help

- **Documentation**: Check existing documentation first
- **Issues**: Search existing issues for similar problems
- **Discussions**: Use GitHub Discussions for questions
- **Chat**: Join our community chat (if available)

## Recognition

Contributors will be recognized in:

- `CHANGELOG.md` for significant contributions
- GitHub contributors list
- Release notes for major features

Thank you for contributing to the Localhost HTTP Server!
