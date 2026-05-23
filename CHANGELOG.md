# Changelog

All notable user-facing changes are documented in this file.

This project follows semantic versioning while it is in early `0.x` releases:
minor versions may still refine behavior, but published install surfaces should
remain honest about platform support.

## 0.1.8 - 2026-05-23

### Added

- Added Linux x64 listener, process, cwd, and process-list collection.
- Added Windows x64 listener and process collection through PowerShell/CIM.
- Added Linux x64 and Windows x64 GitHub Release artifacts.
- Added Linux x64 and Windows x64 npm optional binary packages.
- Added Windows PowerShell install script support.

### Fixed

- Regenerated the cross-platform release with newline-delimited checksum entries
  so `shasum -a 256 -c checksums.txt` verifies every asset cleanly.

## 0.1.7 - 2026-05-23

### Added

- Added Linux x64 listener, process, cwd, and process-list collection.
- Added Windows x64 listener and process collection through PowerShell/CIM.
- Added Linux x64 and Windows x64 GitHub Release artifacts.
- Added Linux x64 and Windows x64 npm optional binary packages.
- Added Windows PowerShell install script support.

### Changed

- Updated README install and platform support sections for macOS, Linux x64,
  and Windows x64.

## 0.1.6 - 2026-05-23

### Added

- Published npm package `@gaossr/kiri` with the public `ports` command.
- Added macOS npm platform packages for Apple Silicon and Intel Macs.
- Published the Homebrew formula at `gaossr/tap/kiri`.
- Added GitHub Actions based npm Trusted Publishing with provenance.

### Changed

- Documented npm, Homebrew, and GitHub Release install paths in the README files.
- Kept macOS as the only fully supported runtime platform for this release.

### Verified

- GitHub Release assets include both macOS tarballs and `checksums.txt`.
- `scripts/install.sh` installs `ports` from the GitHub Release.
- `brew test gaossr/tap/kiri` passes.
- `npm install -g @gaossr/kiri` installs a runnable `ports` command on macOS.

## Earlier releases

Earlier `0.1.x` releases were focused on packaging, release automation, and CLI
stabilization. See GitHub Releases for the complete artifact history.
