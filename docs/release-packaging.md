# DevPorts Release Packaging

This document defines the intended release packaging plan for DevPorts. It does not claim that any npm package, Homebrew formula, tap, or GitHub Release artifact already exists.

## Goals

- Give normal users installation paths that do not require Rust or Cargo.
- Keep `devports` as the primary command, `ports` as the short command, and `whoisonport` as a compatibility detail alias.
- Ship reproducible, versioned binaries from GitHub Releases.
- Avoid asking end users to compile from source.
- Keep Cargo usage limited to maintainers, CI, and local build/test workflows.

## npm Package Plan

The npm package should be the most familiar cross-platform install entry:

```bash
npm install -g devports
```

The package must expose three bins:

```json
{
  "bin": {
    "devports": "./bin/devports.js",
    "ports": "./bin/ports.js",
    "whoisonport": "./bin/whoisonport.js"
  }
}
```

Recommended implementation:

- Build release binaries in CI for each supported target.
- Attach binaries to a GitHub Release.
- Make the npm package install script download the matching platform binary from GitHub Releases.
- Keep `bin/devports`, `bin/ports`, and `bin/whoisonport` as small shims that execute the downloaded binary.
- Do not compile Rust during `npm install`.

Current scaffold:

- `packaging/npm/package.json` declares the future `devports` package and exposes `devports`, `ports`, and `whoisonport` bins.
- `packaging/npm/bin/devports.js`, `packaging/npm/bin/ports.js`, and `packaging/npm/bin/whoisonport.js` are Node shims.
- `packaging/npm/lib/resolve-binary.js` locates a precompiled binary under `vendor/<platform>-<arch>/`.
- The shim fails with a clear message if package artifacts are not bundled yet.
- The package is marked `private` until the release process is wired and the package name is confirmed.

Current local checks:

```bash
cd packaging/npm
npm run check
npm run pack:dry-run
```

Alternative implementation:

- Publish platform-specific npm packages such as `devports-darwin-arm64` and `devports-linux-x64`.
- Make the top-level `devports` package depend on the matching optional package.
- Still expose `devports`, `ports`, and `whoisonport` commands.

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
- Install `ports` and `whoisonport` as additional binaries.
- Run a smoke test such as `system "#{bin}/devports", "--help"` once help output is stable.

Current scaffold:

- `packaging/homebrew/devports.rb.template` is a template only.
- It uses GitHub Release URL placeholders.
- It installs `devports`, `ports`, and `whoisonport`.
- Its `sha256` values are placeholders and must not be used for a real formula.
- No tap is created in this phase.

Formula source:

- The first DevPorts formula should download precompiled macOS artifacts from GitHub Releases.
- A source-build formula is not part of the first release plan because normal users should not need a local Rust build path.

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
- `checksums.txt`

Each archive should include:

- `devports`
- `ports` or install instructions for a `ports` symlink
- `whoisonport` compatibility alias
- `README.md`
- License file when one exists
- Checksums

Release checklist:

- Run `cargo fmt`.
- Run `cargo test`.
- Run macOS smoke checks for `devports`, `ports`, `whoisonport`, `--all`, `--color never`, and `--color always`.
- Build release binaries.
- Generate checksums.
- Attach archives and checksums to the GitHub Release.
- Use those artifacts from npm and Homebrew packaging.

Current scaffold:

- `.github/workflows/release.yml` runs on tag pushes matching `v*`.
- The workflow is macOS-first because macOS is the only platform with real collection support today.
- It builds `aarch64-apple-darwin` and `x86_64-apple-darwin` artifacts.
- It runs `cargo test`.
- It runs `cargo build --release --target <target> --bin devports --bin ports --bin whoisonport`.
- It packages all three binaries plus README into target-specific tarballs.
- It generates per-artifact SHA256 files and combines them into `checksums.txt`.
- It uploads the tarballs and `checksums.txt` to the GitHub Release.

Linux and Windows targets should only be added after their platform collectors are implemented and tested.

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
- Run the release workflow on a real tag and inspect artifacts.
- Wire npm packaging to download or bundle the GitHub Release artifacts.
- Replace Homebrew formula placeholders with real release URLs and SHA256 values.
- Decide how to handle `ports` command conflicts in release notes.
- Decide whether `whoisonport` needs separate release-note mention as a compatibility alias.
- Add stable `--help` output suitable for package smoke tests.
- Add a license file if the repository is distributed publicly.
- Confirm whether Linux and Windows should stay documented as TODO until their collectors are implemented.
