# Contributing to CYCLE

Thank you for your interest in contributing to CYCLE! This document provides guidelines for contributing.

## Development Setup

1. Clone the repository
2. Install Rust (latest stable)
3. Run `cargo test` to ensure everything works
4. Make your changes
5. Submit a pull request

## Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write tests for new functionality
- Update documentation as needed

## Performance

CYCLE is focused on maximum performance. All contributions should:
- Include benchmarks for performance-critical code
- Avoid unnecessary allocations
- Use lock-free algorithms where possible
- Profile performance impact

## License

By contributing, you agree to license your contribution under the same terms as the project.
