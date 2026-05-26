# Changelog

All notable user-facing changes are documented in this file.

This project follows semantic versioning while it is in early `0.x` releases:
minor versions may still refine behavior, but published install surfaces should
remain honest about platform support.

## 0.1.21 - 2026-05-26

### Fixed

- Stopped `ports logs` from falling back to empty macOS system log output when
  the target process writes stdout/stderr to a pipe, terminal, or PTY that
  cannot be replayed from another process.
- Added a clearer restart-with-tee diagnostic for Codex Shell and other
  terminal-backed processes, while still preferring real current log files when
  they can be discovered.

## 0.1.20 - 2026-05-26

### Fixed

- Prevented `ports logs` from treating Codex control files as process log
  files when stdout/stderr is attached to a Codex-managed pipe.
- Added an explicit diagnostic for already-consumed stdout/stderr pipes so
  `ports logs <port|pid> -f` exits with a useful restart-with-tee hint instead
  of tailing an unrelated `.lock` file.

## 0.1.19 - 2026-05-24

### Fixed

- Fixed the macOS/Linux and Windows install scripts so `KIRI_VERSION=0.1.19`
  and `KIRI_VERSION=v0.1.19` both resolve the GitHub Release tag correctly.

## 0.1.18 - 2026-05-24

### Added

- Added README screenshots for the default `ports` overview and colored
  `ports logs -f` output.

### Changed

- Kept common project names on one stable line in the default `ports` table,
  even when the terminal reports a narrower width.
- Switched `ports logs` INFO, source, and success colors from dark green to a
  brighter green for better readability on light terminal backgrounds.
- Scoped `--all` parsing to `ports` and `ports ps`, so unrelated commands reject
  it before doing work.

### Fixed

- Fixed `ports logs` current-process log selection so live stdout/stderr logs
  win over historical project log files.
- Fixed `ports logs` and normal command output so broken pipes exit cleanly.
- Fixed `ports watch` so existing listeners are not reported as new on startup
  and interrupted scans do not create false removals.
- Fixed npm Windows binary resolution so staged Windows runtime checks look for
  `ports.exe`.
- Preserved UTF-8 process commands while keeping stable English process start
  times on macOS and Linux.

## 0.1.17 - 2026-05-24

### Added

- Added the compact Kiri terminal companion to the default `ports` view, with
  the active port count shown beside the companion.

### Changed

- Moved the default `ports` summary into the companion line and removed the
  redundant footer hint from the table view.
- Kept the `Status` column visually balanced by giving status values symmetric
  horizontal padding.
- Removed the Codex Pet assets and generator from the CLI repository; the pet
  package now lives in its own repository.

### Fixed

- Removed backtick characters from runtime command hints such as
  `Run ports --all to show every listener.`

## 0.1.15 - 2026-05-23

### Fixed

- Removed Windows test warnings from Unix-only `ports kill` test imports.

## 0.1.14 - 2026-05-23

### Fixed

- Split project framework log discovery from broad process-redirect log
  detection, so files such as `backend.txt` inside temp-backed test directories
  are not treated as framework logs.

## 0.1.13 - 2026-05-23

### Fixed

- Prevented `ports logs` framework-log fallback tests and runtime discovery from
  climbing into system temp roots such as `/tmp`, avoiding unrelated log files
  outside the project tree.

## 0.1.12 - 2026-05-23

### Added

- Added the Kiri terminal companion status to the default `ports` view.
- Added a reproducible Codex custom pet package for Kiri under
  `assets/codex-pet/kiri`.

### Fixed

- Fixed `ports logs` fallback discovery for project `.dev-logs`, `logs`, and
  `log` directories when the running process writes to project-managed log
  files instead of direct stdout/stderr files.
- Changed log coloring to use a calmer green for INFO, sources, successful
  status values, and logfmt success values, while keeping error states aligned
  with the table `Framework` red.
- Kept non-logger prose such as `complete.` from being colored as a logger
  name in Spring-style log lines.

## 0.1.11 - 2026-05-23

### Added

- Added semantic ANSI coloring for `ports logs` output across common Java,
  Python, Go, Node.js, logfmt, and JSON log formats.

### Changed

- Updated README log examples so continuous log listening uses
  `ports logs <port|pid> -f` explicitly.

## 0.1.10 - 2026-05-23

### Fixed

- Fixed `ports logs` discovery for processes whose stdout or stderr is piped
  into another process that writes the real log file, such as
  `java | tee -a /tmp/nori-backend.log`.
- Fixed empty macOS system log output being treated as useful log output when
  `log show` returns only its table header.

## 0.1.9 - 2026-05-23

### Added

- Added Linux x64 listener, process, cwd, and process-list collection.
- Added Windows x64 listener and process collection through PowerShell/CIM.
- Added Linux x64 and Windows x64 GitHub Release artifacts.
- Added Linux x64 and Windows x64 npm optional binary packages.
- Added Windows PowerShell install script support.

### Fixed

- Normalized Windows-generated checksum lines to Unix line endings when building
  `checksums.txt`, so `shasum -a 256 -c checksums.txt` works on macOS and Linux.

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
