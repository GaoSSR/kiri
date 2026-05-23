"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { createRequire } = require("node:module");
const test = require("node:test");

const {
  binaryPath,
  platformKey,
  platformPackageName,
  resolveBinary,
} = require("../lib/resolve-binary");

test("maps supported runtimes to npm platform package aliases", () => {
  assert.equal(
    platformKey({ platform: "darwin", arch: "arm64" }),
    "darwin-arm64"
  );
  assert.equal(platformKey({ platform: "darwin", arch: "x64" }), "darwin-x64");
  assert.equal(platformKey({ platform: "linux", arch: "x64" }), "linux-x64");
  assert.equal(platformKey({ platform: "win32", arch: "x64" }), "win32-x64");
  assert.equal(platformPackageName("darwin-arm64"), "@gaossr/kiri-darwin-arm64");
  assert.equal(platformPackageName("darwin-x64"), "@gaossr/kiri-darwin-x64");
  assert.equal(platformPackageName("linux-x64"), "@gaossr/kiri-linux-x64");
  assert.equal(platformPackageName("win32-x64"), "@gaossr/kiri-win32-x64");
});

test("resolves ports from an installed optional platform package first", () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "kiri-npm-resolve-"));
  const packageRoot = path.join(tempRoot, "root-package");
  const dependencyRoot = path.join(
    packageRoot,
    "node_modules",
    "@gaossr",
    "kiri-darwin-arm64"
  );
  const dependencyBinary = path.join(
    dependencyRoot,
    "vendor",
    "darwin-arm64",
    "ports"
  );

  fs.mkdirSync(path.dirname(dependencyBinary), { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), "{}\n");
  fs.writeFileSync(path.join(dependencyRoot, "package.json"), "{}\n");
  fs.writeFileSync(dependencyBinary, "#!/bin/sh\n");

  const resolved = resolveBinary("ports", {
    packageRoot,
    requireFn: createRequire(path.join(packageRoot, "package.json")),
    runtime: { platform: "darwin", arch: "arm64" },
  });

  assert.equal(resolved, fs.realpathSync(dependencyBinary));
});

test("returns null when the platform has no npm binary package", () => {
  assert.equal(platformPackageName("linux-arm64"), undefined);
  assert.equal(
    resolveBinary("ports", {
      runtime: { platform: "linux", arch: "arm64" },
    }),
    null
  );
});

test("falls back to local vendor binaries for staged package verification", () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "kiri-npm-local-"));
  const localBinary = path.join(tempRoot, "vendor", "darwin-x64", "ports");
  fs.mkdirSync(path.dirname(localBinary), { recursive: true });
  fs.writeFileSync(localBinary, "#!/bin/sh\n");

  assert.equal(
    binaryPath("ports", {
      packageRoot: tempRoot,
      runtime: { platform: "darwin", arch: "x64" },
    }),
    localBinary
  );
  assert.equal(
    resolveBinary("ports", {
      packageRoot: tempRoot,
      requireFn: createRequire(__filename),
      runtime: { platform: "darwin", arch: "x64" },
    }),
    localBinary
  );
});
