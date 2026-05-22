# DevPorts Release Packaging

This document defines the intended release packaging plan for DevPorts. It does not claim that any npm package, Homebrew formula, tap, or GitHub Release artifact already exists.

## Goals

- Give normal users installation paths that do not require Rust or Cargo.
- Keep `devports` as the primary command and `ports` as the short command.
- Ship reproducible, versioned binaries from GitHub Releases.
- Avoid asking end users to compile from source.
- Keep Cargo usage limited to maintainers, CI, and local build/test workflows.

## npm Package Plan

The npm package should be the most familiar cross-platform install entry:

```bash
npm install -g devports
```

The package must expose two bins:

```json
{
  "bin": {
    "devports": "bin/devports",
    "ports": "bin/ports"
  }
}
```

Recommended implementation:

- Build release binaries in CI for each supported target.
- Attach binaries to a GitHub Release.
- Make the npm package install script download the matching platform binary from GitHub Releases.
- Keep `bin/devports` and `bin/ports` as small shims that execute the downloaded binary.
- Do not compile Rust during `npm install`.

Alternative implementation:

- Publish platform-specific npm packages such as `devports-darwin-arm64` and `devports-linux-x64`.
- Make the top-level `devports` package depend on the matching optional package.
- Still expose both `devports` and `ports` commands.

Open decisions before npm release:

- Final npm package name availability.
- Supported platform matrix for the first public release.
- Binary download URL layout.
- Install script behavior when the current platform is unsupported.
- Whether `ports` should be installed by default or documented as a possible command-name conflict.

## Homebrew Formula Plan

DevPorts is a CLI tool, so Homebrew distribution should use a formula, not a cask.

Intended install command after publication:

```bash
brew install devports
```

If distribution starts through a tap:

```bash
brew install <tap-owner>/tap/devports
```

Recommended formula behavior:

- Download a versioned GitHub Release artifact.
- Install the `devports` binary into `bin`.
- Install `ports` as a second binary or symlink to the same executable.
- Run a smoke test such as `system "#{bin}/devports", "--help"` once help output is stable.

Formula source options:

- Binary formula: download precompiled macOS artifacts from GitHub Releases.
- Source formula: download a source tarball and build with Cargo in Homebrew.

The binary formula is preferable for normal users because it matches the npm plan and avoids local Rust build time. A source formula is acceptable if Homebrew policy or distribution constraints require it.

Open decisions before Homebrew release:

- Homebrew core vs tap.
- Formula name and tap owner.
- SHA256 checksums for release artifacts.
- Whether the formula installs `ports` directly or as a symlink.

## GitHub Releases Plan

GitHub Releases should be the source of truth for versioned binaries.

Recommended artifacts:

- `devports-aarch64-apple-darwin.tar.gz`
- `devports-x86_64-apple-darwin.tar.gz`
- Future Linux artifacts after Linux support is real.
- Future Windows artifacts after Windows support is real.

Each archive should include:

- `devports`
- `ports` or install instructions for a `ports` symlink
- `README.md`
- License file when one exists
- Checksums

Release checklist:

- Run `cargo fmt`.
- Run `cargo test`.
- Run macOS smoke checks for `devports`, `ports`, `--all`, `--color never`, and `--color always`.
- Build release binaries.
- Generate checksums.
- Attach archives and checksums to the GitHub Release.
- Use those artifacts from npm and Homebrew packaging.

## Why Cargo Is Not The User Install Path

Cargo is useful for maintainers because it validates that the Rust crate builds, tests pass, and release binaries can be produced in CI.

It is not the right primary user install path because:

- It requires Rust and Cargo on the user's machine.
- It can compile from source instead of installing a signed or versioned release artifact.
- It does not solve common PATH and command-name conflict issues for normal users.
- It feels like a developer workflow rather than a product install experience.

For README usage, Cargo commands belong only in maintainer build and test sections, not in the user installation section.

## Missing Before Public Release

- Decide final npm package ownership and package name.
- Decide Homebrew core vs tap.
- Add release automation for macOS binaries.
- Add GitHub Release archives and checksums.
- Add npm package wrapper or platform package structure.
- Add Homebrew formula.
- Decide how to handle `ports` command conflicts in release notes.
- Add stable `--help` output suitable for package smoke tests.
- Add a license file if the repository is distributed publicly.
- Confirm whether Linux and Windows should stay documented as TODO until their collectors are implemented.
