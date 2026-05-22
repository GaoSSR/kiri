# Kiri npm Packaging

This directory packages Kiri for npm with precompiled macOS binaries.

After the npm package is published, users install it with:

```bash
npm install -g @gaossr/kiri
```

The root package name is `@gaossr/kiri`. It exposes one command:

- `ports`

The bin shim in `bin/` does not call Cargo and does not compile Rust locally. It locates the matching optional platform package and executes the precompiled binary from that package.

The package follows the same optional-dependency alias pattern used by `@openai/codex`: the installable root package depends on platform aliases, while the underlying published package name remains `@gaossr/kiri` with platform-specific prerelease versions.

Root package optional dependencies:

```json
{
  "@gaossr/kiri-darwin-arm64": "npm:@gaossr/kiri@<version>-darwin-arm64",
  "@gaossr/kiri-darwin-x64": "npm:@gaossr/kiri@<version>-darwin-x64"
}
```

Platform package vendor layout:

```text
vendor/
  darwin-arm64/
    ports
  darwin-x64/
    ports
```

If the matching binary is missing, the shim prints a clear error explaining that the npm package artifacts are missing and should be reinstalled.

Local checks:

```bash
npm run check
npm run test
npm run pack:dry-run
```

Build release tarballs from GitHub Release assets:

```bash
node scripts/build-packages.js \
  --version 0.1.6 \
  --release-dir ../../dist \
  --output-dir ../../dist/npm
```

Publish order, after verifying tarball contents and npm authentication:

```bash
npm publish ../../dist/npm/kiri-npm-darwin-arm64-0.1.6.tgz --tag darwin-arm64 --access public --registry https://registry.npmjs.org/
npm publish ../../dist/npm/kiri-npm-darwin-x64-0.1.6.tgz --tag darwin-x64 --access public --registry https://registry.npmjs.org/
npm publish ../../dist/npm/kiri-npm-0.1.6.tgz --access public --registry https://registry.npmjs.org/
```
