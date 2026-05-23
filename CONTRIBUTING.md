# Contributing to Kiri

Thanks for taking the time to improve Kiri.

Kiri is currently macOS-first. Linux and Windows are planned, but they should not
be documented as supported until their collectors and release artifacts are ready.

## Product boundaries

- Product name: `Kiri`
- Public command: `ports`
- Cargo package: `kiri`
- npm package: `@gaossr/kiri`
- Homebrew formula: `gaossr/tap/kiri`
- Current supported runtime platform: macOS

Do not reintroduce legacy public command names. Keep user-facing install
instructions focused on npm, Homebrew, and the GitHub Release install script.

## Development setup

Install Rust using rustup, then run:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Useful local commands:

```bash
cargo run --bin ports
cargo run --bin ports -- --all
cargo run --bin ports -- ps
cargo run --bin ports -- logs 3000 --lines 20
```

For npm packaging work:

```bash
npm --prefix packaging/npm run check
npm --prefix packaging/npm run test
```

For Homebrew formula work:

```bash
brew audit --formula gaossr/tap/kiri
brew test gaossr/tap/kiri
```

## Pull request expectations

- Keep changes focused on the reported behavior or documented feature.
- Add or update tests for parser, formatter, renderer, release, or packaging
  behavior whenever the change can regress.
- Preserve the public `ports` command and the current macOS support boundary.
- Run the relevant verification commands before opening a pull request.
- For release and packaging changes, include the exact install path or artifact
  that was verified.

## Release notes

Release-facing changes should update `CHANGELOG.md` and keep `README.md` /
`README_CN.md` aligned when install commands, support status, or package names
change.
