# Contributing to Audio Epistles

Thank you for your interest in contributing to Audio Epistles! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)
- [Feature Requests](#feature-requests)

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow:

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Respect differing viewpoints and experiences
- Accept responsibility and apologize when mistakes are made

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- Rust (1.85.1 or later) - [Install via rustup](https://rustup.rs/)
- FFmpeg - `brew install ffmpeg` (macOS) or `apt-get install ffmpeg` (Linux)
- yt-dlp - `brew install yt-dlp` (macOS) or `pip install yt-dlp` (Linux)
- ChromeDriver - `brew install chromedriver` (macOS)
- Git - For version control

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/audio-epistles.git
   cd audio-epistles
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/chornge/audio-epistles.git
   ```

### Setup Development Environment

1. Copy the example environment file:

   ```bash
   cp .env.example .env
   ```

2. Configure your `.env` file with your credentials (for testing)

3. Build the project:

   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Branching Strategy

- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates

### Creating a New Branch

```bash
# Update your local repository
git checkout develop
git pull upstream develop

# Create a new branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### Making Changes

1. Make your changes in your feature branch
2. Write or update tests as needed
3. Update documentation if necessary
4. Ensure all tests pass: `cargo test`
5. Run linting: `cargo clippy -- -D warnings`
6. Format code: `cargo fmt`

### Commit Messages

Follow the conventional commits specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**

```
feat(webdriver): add retry logic for failed uploads

fix(episode): correct timestamp parsing for single-digit hours

docs(readme): update installation instructions
```

## Coding Standards

### Rust Style Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code
- Run `cargo clippy` and address all warnings
- Write idiomatic Rust code
- Use meaningful variable and function names

### Code Quality

- **Error Handling**: Use `Result<T, E>` and the `?` operator; avoid `unwrap()` and `expect()` except in tests
- **Comments**: Add doc comments (`///`) for public functions and modules
- **Complexity**: Keep functions small and focused (under 50 lines when possible)
- **Dependencies**: Justify new dependencies in PR description

### Documentation

- Add doc comments for all public APIs
- Include examples in doc comments where helpful
- Update README.md for user-facing changes
- Update CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/) format

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests

- Write unit tests in the same file as the code
- Write integration tests in `tests/` directory
- Aim for high test coverage of critical paths
- Test both success and error cases

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp("1:30"), 90);
        assert_eq!(parse_timestamp("1:05:30"), 3930);
    }
}
```

## Submitting Changes

### Pull Request Process

1. **Update your branch** with latest upstream changes:

   ```bash
   git checkout develop
   git pull upstream develop
   git checkout your-branch
   git rebase develop
   ```

2. **Push to your fork:**

   ```bash
   git push origin your-branch
   ```

3. **Create a Pull Request** on GitHub:
   - Base: `develop`
   - Compare: `your-branch`
   - Fill out the PR template

### Pull Request Guidelines

- Provide clear description of changes
- Reference related issues (e.g., "Closes #123")
- Include screenshots for UI changes
- Ensure all CI checks pass
- Request review from maintainers
- Be responsive to feedback

### PR Title Format

```
<type>: <short description>
```

Examples:

- `feat: add support for video chapters`
- `fix: resolve database locking issue`
- `docs: improve setup instructions`

### Review Process

- PRs require at least one approval
- Address all review comments
- Keep PRs focused and reasonably sized
- Be patient and respectful during review

## Reporting Issues

### Before Submitting an Issue

1. Check existing issues to avoid duplicates
2. Update to the latest version
3. Gather relevant information:
   - OS and version
   - Rust version (`rustc --version`)
   - Error messages and logs
   - Steps to reproduce

### Issue Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:

1. Run command '...'
2. See error

**Expected behavior**
What you expected to happen.

**Environment:**

- OS: [e.g., macOS 14.5]
- Rust version: [e.g., 1.85.1]
- Audio Epistles version: [e.g., 2.2.6]

**Additional context**
Any other relevant information.
```

## Feature Requests

We welcome feature requests! Please:

1. Check if the feature is already requested
2. Describe the use case clearly
3. Explain why this feature would be useful
4. Provide examples if possible

**Feature Request Template:**

```markdown
**Is your feature request related to a problem?**
A clear description of the problem.

**Describe the solution you'd like**
What you want to happen.

**Describe alternatives you've considered**
Other solutions you've thought about.

**Additional context**
Any other relevant information.
```

## Development Tips

### Debugging

- Use `RUST_LOG=debug cargo run` for verbose logging
- Use `RUST_BACKTRACE=1 cargo run` for stack traces
- Add `dbg!()` macro for quick debugging
- Use `cargo run -- --help` for CLI options

### Common Issues

**ChromeDriver not starting:**

- Check ChromeDriver is in PATH
- Verify Chrome version matches ChromeDriver version
- On macOS: `xattr -d com.apple.quarantine $(which chromedriver)`

**Database locked errors:**

- Ensure only one instance is running
- Check file permissions

**FFmpeg errors:**

- Verify FFmpeg is installed: `ffmpeg -version`
- Check input file exists and is valid

## Questions?

If you have questions not covered here:

- Open a [GitHub Discussion](https://github.com/chornge/audio-epistles/discussions)
- Email the maintainers (see [README.md](./README.md))
- Check the [README.md](./README.md) for more information

## License

By contributing to Audio Epistles, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Audio Epistles! Your efforts help make this project better for everyone.
