# Contributing to GenXLink

Thank you for your interest in contributing to GenXLink! This document provides guidelines for contributing to the project.

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Maintain professional communication

## How to Contribute

### Reporting Bugs

1. Check existing issues first
2. Use the bug report template
3. Include:
   - Operating system and version
   - GenXLink version
   - Steps to reproduce
   - Expected vs actual behavior
   - Logs and screenshots

### Suggesting Features

1. Check if feature already requested
2. Use the feature request template
3. Explain the use case
4. Describe the proposed solution
5. Consider alternatives

### Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write/update tests
5. Update documentation
6. Submit PR with clear description

## Development Setup

See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for detailed setup instructions.

## Coding Standards

### Rust Code

- Follow Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write documentation for public APIs
- Add tests for new features

### Commit Messages

Format:
```
type(scope): subject

body

footer
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Example:
```
feat(client): add multi-monitor support

Implement screen selection for multiple monitors
using DXGI output enumeration.

Closes #123
```

### Code Review

All PRs require:
- Passing CI/CD checks
- Code review approval
- No merge conflicts
- Updated documentation

## Testing

- Write unit tests for new code
- Update integration tests if needed
- Test on multiple platforms
- Verify no performance regression

## Documentation

- Update README if needed
- Add/update API documentation
- Include code examples
- Update CHANGELOG

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

## Questions?

- Open a discussion on GitHub
- Join our community chat
- Email: dev@genxis.com

Thank you for contributing!
