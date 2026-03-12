# Contributing to ContextPaste

Thank you for your interest in contributing to ContextPaste!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/contextpaste.git`
3. Install prerequisites:
   - Rust 1.77+ (`rustup update stable`)
   - Node.js 18+ and pnpm
   - Platform-specific deps: see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
4. Install dependencies: `pnpm install`
5. Run in dev mode: `cargo tauri dev`

## Development Workflow

1. Create a branch: `git checkout -b feature/your-feature`
2. Make changes
3. Run checks:
   ```bash
   cargo check && cargo clippy -- -D warnings && cargo test
   pnpm tsc --noEmit && pnpm vitest run
   ```
4. Commit with conventional format: `feat(clipboard): add image support`
5. Push and open a PR

## Code Guidelines

### Rust
- No `.unwrap()` in production code — use `?` or proper error handling
- Use `thiserror` for custom error types
- Run `cargo fmt` before committing
- All public functions need doc comments

### TypeScript/React
- No `any` types — everything explicitly typed
- Tailwind CSS only for styling
- All Tauri IPC calls go through `src/lib/tauri.ts` wrappers
- Component files use PascalCase, other files use camelCase

### Commits
Follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat(module): description` — new feature
- `fix(module): description` — bug fix
- `refactor(module): description` — code change that neither fixes a bug nor adds a feature
- `test(module): description` — adding or correcting tests
- `docs: description` — documentation changes

## Reporting Issues

Use GitHub Issues with these labels:
- `bug` — something isn't working
- `feature` — feature request
- `security` — security-related issue (please use private disclosure)

## License

By contributing, you agree that your contributions will be licensed under the GPL v3 License.
