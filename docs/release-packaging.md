# Kiri Release Packaging

This document defines the intended release packaging plan for Kiri. It does not claim that any npm package, Homebrew formula, tap, install script, or GitHub Release artifact already exists.

## Naming

- Product name: Kiri.
- Repository: `gaossr/kiri`.
- Cargo package: `kiri`.
- npm package: `@gaossr/kiri`.
- Homebrew tap formula: `gaossr/tap/kiri`.
- Installed command: `ports`.
- GitHub Release artifacts: `kiri-<target>.tar.gz`.

The unscoped npm package name `kiri` is occupied, so the npm distribution uses the scoped package `@gaossr/kiri`. This keeps the Kiri brand intact without introducing a split name such as a separate ports-branded package. Kiri is the product, `@gaossr/kiri` is the npm distribution name, and `ports` is the command.

## Goals

- Give normal users installation paths that do not require Rust or Cargo.
- Support curl, npm, and Homebrew installation channels.
- Ship one public command: `ports`.
- Ship reproducible, versioned binaries from GitHub Releases.
- Avoid asking end users to compile from source.
- Keep Cargo usage limited to maintainers, CI, and local build/test workflows.

## Install Channels

Planned install script:

```bash
curl -fsSL https://raw.githubusercontent.com/gaossr/kiri/main/scripts/install.sh | bash
```

Planned npm install:

```bash
npm install -g @gaossr/kiri
```

Planned Homebrew install:

```bash
brew install gaossr/tap/kiri
```

Planned Windows PowerShell entry:

```powershell
irm https://raw.githubusercontent.com/gaossr/kiri/main/scripts/install.ps1 | iex
```

Windows support is not available yet. The PowerShell script is an entry scaffold and must report that Windows artifacts and collectors have not shipped.

## GitHub Releases Plan

GitHub Releases should be the source of truth for versioned binaries.

Recommended artifacts for the macOS-first release:

- `kiri-aarch64-apple-darwin.tar.gz`
- `kiri-x86_64-apple-darwin.tar.gz`
- `checksums.txt`

Each archive should include:

- `ports`
- `README.md`
- License file when one exists
- Checksums

Current scaffold:

- `.github/workflows/release.yml` runs on tag pushes matching `v*`.
- The workflow is macOS-first because macOS is the only platform with real collection support today.
- It builds `aarch64-apple-darwin` and `x86_64-apple-darwin` artifacts.
- It runs `cargo test`.
- It runs `cargo build --release --target <target> --bin ports`.
- It packages the `ports` binary plus README into target-specific tarballs.
- It generates per-artifact SHA256 files and combines them into `checksums.txt`.
- It uploads the tarballs and `checksums.txt` to the GitHub Release.

Linux and Windows targets should only be added after their platform collectors are implemented and tested.

## npm Package Plan

The npm package is `@gaossr/kiri` and exposes only the `ports` bin:

```json
{
  "bin": {
    "ports": "./bin/ports.js"
  }
}
```

Recommended implementation:

- Build release binaries in CI for each supported target.
- Attach binaries to a GitHub Release.
- Make the npm package install script download the matching platform binary from GitHub Releases, or bundle platform-specific binaries through optional packages.
- Keep `bin/ports` as a small shim that executes the downloaded or bundled binary.
- Do not compile Rust during `npm install`.

Current scaffold:

- `packaging/npm/package.json` declares the future `@gaossr/kiri` package and exposes the `ports` bin.
- `packaging/npm/bin/ports.js` is the Node shim.
- `packaging/npm/lib/resolve-binary.js` locates a precompiled binary under `vendor/<platform>-<arch>/`.
- The shim fails with a clear message if package artifacts are not bundled yet or if the current platform is unsupported.
- The package is marked `private` until the release process is wired and the package name is confirmed.

Current local checks:

```bash
cd packaging/npm
npm run check
npm run pack:dry-run
```

Open decisions before npm release:

- Final npm package ownership and publish permissions for `@gaossr/kiri`.
- Binary download URL layout.
- Install script behavior when the current platform is unsupported.
- Release-note handling for possible `ports` command-name conflicts.

## Homebrew Formula Plan

Kiri is a CLI tool, so Homebrew distribution should use a formula, not a cask.

Intended tap install command after publication:

```bash
brew install gaossr/tap/kiri
```

If Kiri later enters Homebrew core, the install command can become:

```bash
brew install kiri
```

Recommended formula behavior:

- Download a versioned GitHub Release artifact.
- Install the `ports` binary into `bin`.
- Run a smoke test such as `system "#{bin}/ports", "--help"` once help output is stable.

Current scaffold:

- `packaging/homebrew/kiri.rb.template` is a template only.
- It uses GitHub Release URL placeholders.
- It installs `ports`.
- Its `sha256` values are placeholders and must not be used for a real formula.
- No tap is created in this phase.

## Platform Expansion Requirements

Linux and Windows need more than packaging before they can be claimed as supported:

- Platform collector implementation.
- Process info and cwd enrichment.
- Safe kill/logs behavior validated on that platform.
- GitHub Actions matrix entries.
- Release artifacts for each target.
- npm resolver and vendor mapping for each target.
- `install.sh` and `install.ps1` download paths for each target.

Until that work is done, Linux and Windows installers must clearly report unsupported or not available.

## Why Cargo Is Not The User Install Path

Cargo is useful for maintainers because it validates that the Rust crate builds, tests pass, and release binaries can be produced in CI.

It is not the right primary user install path because:

- It requires Rust and Cargo on the user's machine.
- It can compile from source instead of installing a versioned release artifact.
- It does not solve common PATH and command-name conflict issues for normal users.
- It feels like a developer workflow rather than a product install experience.

For README usage, Cargo commands belong only in maintainer build and test sections, not in the user installation section.

## Missing Before Public Release

- Publish real GitHub Release artifacts for macOS.
- Wire npm packaging to download or bundle the GitHub Release artifacts.
- Replace Homebrew formula placeholders with real release URLs and SHA256 values.
- Decide how to handle `ports` command conflicts in release notes.
- Add stable `--help` output suitable for package smoke tests.
- Add a license file if the repository is distributed publicly.
- Add the final logo file at `assets/kiri-logo.png`.
- Confirm whether Linux and Windows should stay documented as TODO until their collectors are implemented.
