# Kiri npm Packaging

This directory is the npm packaging scaffold for Kiri. It is not published yet.

The future npm package should install with:

```bash
npm install -g kiri
```

The package exposes one command:

- `ports`

The bin shim in `bin/` does not call Cargo and does not compile Rust locally. It locates a precompiled binary under `vendor/<platform>-<arch>/` and executes it.

Expected vendor layout after release packaging is wired:

```text
vendor/
  darwin-arm64/
    ports
  darwin-x64/
    ports
```

If the matching binary is missing, the shim prints a clear error explaining that npm package artifacts are not bundled yet.

Local checks:

```bash
npm run check
npm run pack:dry-run
```
