# Kiri npm Packaging

This directory packages Kiri for npm with precompiled macOS, Linux x64, and Windows x64 binaries.

Users install it with:

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
  "@gaossr/kiri-darwin-x64": "npm:@gaossr/kiri@<version>-darwin-x64",
  "@gaossr/kiri-linux-x64": "npm:@gaossr/kiri@<version>-linux-x64",
  "@gaossr/kiri-win32-x64": "npm:@gaossr/kiri@<version>-win32-x64"
}
```

Platform package vendor layout:

```text
vendor/
  darwin-arm64/
    ports
  darwin-x64/
    ports
  linux-x64/
    ports
  win32-x64/
    ports.exe
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
  --version 0.1.7 \
  --release-dir ../../dist \
  --output-dir ../../dist/npm
```

Publishing uses npm Trusted Publishing through GitHub Actions OIDC. Do not store
`NPM_TOKEN` in GitHub Actions secrets for the normal release path.

Required npm trusted publisher configuration for `@gaossr/kiri`:

- Provider: GitHub Actions
- Repository: `GaoSSR/kiri`
- Workflow filename: `npm-publish.yml`
- Allowed action: `npm publish`

The equivalent npm CLI setup command is:

```bash
npm trust github @gaossr/kiri --file npm-publish.yml --repo GaoSSR/kiri --allow-publish
```

This requires npm CLI 11.10.0 or newer, write access to `@gaossr/kiri`, and
2FA enabled on the npm account.

After the Release workflow has uploaded and checksummed the npm tarballs, publish
the already-built artifacts with:

```bash
gh workflow run "Publish npm" --repo GaoSSR/kiri --ref main -f version=<version>
```
