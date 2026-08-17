# Contributing to KnowteQuiz

Thank you for your interest in contributing! This guide will help you get started.

## Development Setup

### Prerequisites

- [Node.js](https://nodejs.org/) v18+
- [Rust](https://rustup.rs/) (Cargo)
- Git

### Getting Started

```bash
git clone https://github.com/yourname/knowtequiz.git
cd knowtequiz
node setup.cjs          # Interactive installer
```

Or manually:

```bash
npm install
npm run build           # Build frontend
```

### Development Commands

| Task | Command |
|------|---------|
| Frontend dev server | `npm run dev` |
| Desktop dev (hot-reload) | `npm run tauri:dev` |
| Web server | `npm run web` |
| Type-check | `npx vue-tsc --noEmit` |
| Frontend tests | `npm run test:unit` |
| Rust tests | `cd src-tauri && cargo test` |
| Rust lint | `cd src-tauri && cargo clippy -- -D warnings` |
| Format code | `npm run format` |

## Code Style

- **TypeScript/Vue**: Prettier (single quotes, semicolons, 120 print width)
- **Rust**: `cargo clippy` with `-D warnings`
- Pre-commit hooks run Prettier automatically on staged files

## Project Structure

```
src/                    # Vue 3 frontend
├── components/         # Vue components (Explorer, Reader, Panel, Quiz)
├── services/           # API layer (Tauri invoke / HTTP fetch)
├── stores/             # Pinia stores
├── types/              # TypeScript types
└── utils/              # Utility functions

src-tauri/              # Rust backend
├── src/
│   ├── commands/       # Tauri command handlers
│   ├── services/       # Business logic (quiz, storage, notes)
│   └── models/         # Rust data structures
└── Cargo.toml
```

## Testing

All contributions should include tests:

- **Frontend**: Add tests in `*.test.ts` files next to the code being tested. Run with `npm run test:unit`.
- **Rust**: Add `#[test]` functions in `mod tests` blocks. Run with `cd src-tauri && cargo test`.
- **Aim for coverage**: Test both happy paths and edge cases (error handling, empty inputs, boundary conditions).

## Commit Convention

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
type(scope): description

feat(quiz): add multi-select answer support
fix(storage): handle corrupt JSON gracefully
docs(readme): update installation instructions
test(mistake): add filter edge case tests
chore(ci): expand smoke test coverage
```

Common types: `feat`, `fix`, `docs`, `test`, `chore`, `refactor`, `style`, `ci`

## Pull Request Process

1. **Fork** the repository and create a feature branch from `main`.
2. **Make changes** with tests and ensure all checks pass:
   ```bash
   npx vue-tsc --noEmit          # Type-check
   npm run test:unit             # Frontend tests
   cd src-tauri && cargo test    # Rust tests
   cd src-tauri && cargo clippy -- -D warnings  # Lint
   ```
3. **Commit** with conventional commit messages.
4. **Push** and open a Pull Request against `main`.
5. Describe **what** changed and **why** in the PR description.

## Reporting Issues

- Use GitHub Issues for bug reports and feature requests.
- For bugs, include: OS, steps to reproduce, expected vs actual behavior.
- For features, describe the use case and your proposed solution.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
