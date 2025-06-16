# Contributing to Lab Manager

Thank you for your interest in contributing to Lab Manager! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## License

This project is licensed under the MIT License. By contributing to this project, you agree that your contributions will be licensed under the same MIT License.

### License Headers

All new source files should include the following license header:

```rust
// MIT License
//
// Copyright (c) 2024 Lab Manager
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
```

## Development Workflow

1. **Fork the Repository**
   - Create a fork of the repository on GitHub
   - Clone your fork locally

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Follow the coding standards
   - Write tests for new features
   - Update documentation
   - Keep commits atomic and well-described

4. **Run Tests**
   ```bash
   # Backend tests
   cargo test
   
   # Frontend tests
   cd frontend
   npm test
   ```

5. **Submit a Pull Request**
   - Push your changes to your fork
   - Create a pull request against the main repository
   - Fill out the pull request template

## Coding Standards

### Rust (Backend)

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Use `rustfmt` for code formatting
- Run `clippy` for linting
- Document public APIs with rustdoc comments

### TypeScript/React (Frontend)

- Follow the [TypeScript Style Guide](https://google.github.io/styleguide/tsguide.html)
- Use ESLint and Prettier for code formatting
- Write component documentation using JSDoc
- Follow React best practices and hooks guidelines

## Documentation

- Update relevant documentation for all changes
- Follow the existing documentation structure
- Include code examples where appropriate
- Keep documentation up-to-date with code changes

## Testing

### Backend Tests

- Write unit tests for new functionality
- Include integration tests for API endpoints
- Test error cases and edge conditions
- Maintain test coverage above 80%

### Frontend Tests

- Write unit tests for components
- Include integration tests for user flows
- Test responsive design
- Maintain test coverage above 80%

## Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

## Pull Request Process

1. **Description**
   - Clearly describe the changes
   - Link related issues
   - Include screenshots for UI changes

2. **Review**
   - Address review comments
   - Keep the PR focused and small
   - Update the PR as needed

3. **Merge**
   - Squash commits if requested
   - Ensure CI passes
   - Get required approvals

## Development Setup

1. **Backend Setup**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install dependencies
   cargo install sqlx-cli
   ```

2. **Frontend Setup**
   ```bash
   cd frontend
   npm install
   ```

3. **Database Setup**
   ```bash
   # Start PostgreSQL
   docker-compose up -d db
   
   # Run migrations
   cargo sqlx migrate run
   ```

## Need Help?

- Open an issue for bugs or feature requests
- Join our community chat
- Check the [Documentation Center](docs/README.md)

## Acknowledgments

Thank you for contributing to Lab Manager! Your contributions help make this project better for everyone. 
