# DevPorts npm Packaging

This directory is the npm packaging scaffold for DevPorts. It is not published yet.

The future npm package should install with:

```bash
npm install -g devports
```

The package exposes three commands:

- `devports`
- `ports`
- `whoisonport`

The bin shims in `bin/` do not call Cargo and do not compile Rust locally. They locate precompiled binaries under `vendor/<platform>-<arch>/` and execute them.

Expected vendor layout after release packaging is wired:

```text
vendor/
  darwin-arm64/
    devports
    ports
    whoisonport
  darwin-x64/
    devports
    ports
    whoisonport
```

If the matching binary is missing, the shim prints a clear error explaining that npm package artifacts are not bundled yet.

Local checks:

```bash
npm run check
npm run pack:dry-run
```
