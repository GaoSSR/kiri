"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const test = require("node:test");

const packageRoot = path.resolve(__dirname, "..");
const buildScript = path.join(packageRoot, "scripts", "build-packages.js");

function createReleaseArchive(releaseDir, target, options = {}) {
  const staging = fs.mkdtempSync(path.join(os.tmpdir(), `kiri-${target}-`));
  fs.writeFileSync(path.join(staging, "README.md"), "# Kiri\n");
  fs.writeFileSync(path.join(staging, "LICENSE"), "Apache-2.0\n");
  const binaryName = options.binaryName || "ports";
  const binaryPath = path.join(staging, binaryName);
  fs.writeFileSync(binaryPath, "#!/bin/sh\necho ports\n");
  fs.chmodSync(binaryPath, 0o755);
  if (options.archiveExt === ".zip") {
    execFileSync("zip", ["-qr", path.join(releaseDir, `kiri-${target}.zip`), "."], {
      cwd: staging,
    });
  } else {
    execFileSync("tar", [
      "-czf",
      path.join(releaseDir, `kiri-${target}.tar.gz`),
      "-C",
      staging,
      ".",
    ]);
  }
}

function extractPackageJson(tarballPath, tempRoot) {
  const extractDir = fs.mkdtempSync(path.join(tempRoot, "extract-"));
  execFileSync("tar", ["-xzf", tarballPath, "-C", extractDir]);
  return JSON.parse(
    fs.readFileSync(path.join(extractDir, "package", "package.json"), "utf8")
  );
}

function listTarball(tarballPath) {
  return execFileSync("tar", ["-tzf", tarballPath], { encoding: "utf8" })
    .trim()
    .split("\n")
    .sort();
}

test("builds publishable root and platform npm tarballs from release assets", () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "kiri-npm-build-"));
  const releaseDir = path.join(tempRoot, "release");
  const outputDir = path.join(tempRoot, "npm");
  fs.mkdirSync(releaseDir);
  fs.mkdirSync(outputDir);

  createReleaseArchive(releaseDir, "aarch64-apple-darwin");
  createReleaseArchive(releaseDir, "x86_64-apple-darwin");
  createReleaseArchive(releaseDir, "x86_64-unknown-linux-musl");
  createReleaseArchive(releaseDir, "x86_64-pc-windows-msvc", {
    archiveExt: ".zip",
    binaryName: "ports.exe",
  });

  execFileSync(process.execPath, [
    buildScript,
    "--version",
    "9.9.9",
    "--release-dir",
    releaseDir,
    "--output-dir",
    outputDir,
  ]);

  const rootTarball = path.join(outputDir, "kiri-npm-9.9.9.tgz");
  const armTarball = path.join(outputDir, "kiri-npm-darwin-arm64-9.9.9.tgz");
  const x64Tarball = path.join(outputDir, "kiri-npm-darwin-x64-9.9.9.tgz");
  const linuxTarball = path.join(outputDir, "kiri-npm-linux-x64-9.9.9.tgz");
  const windowsTarball = path.join(outputDir, "kiri-npm-win32-x64-9.9.9.tgz");

  assert.ok(fs.existsSync(rootTarball));
  assert.ok(fs.existsSync(armTarball));
  assert.ok(fs.existsSync(x64Tarball));
  assert.ok(fs.existsSync(linuxTarball));
  assert.ok(fs.existsSync(windowsTarball));

  const rootPackage = extractPackageJson(rootTarball, tempRoot);
  assert.equal(rootPackage.name, "@gaossr/kiri");
  assert.equal(rootPackage.version, "9.9.9");
  assert.equal(rootPackage.private, undefined);
  assert.equal(rootPackage.repository.url, "git+https://github.com/GaoSSR/kiri.git");
  assert.deepEqual(rootPackage.optionalDependencies, {
    "@gaossr/kiri-darwin-arm64": "npm:@gaossr/kiri@9.9.9-darwin-arm64",
    "@gaossr/kiri-darwin-x64": "npm:@gaossr/kiri@9.9.9-darwin-x64",
    "@gaossr/kiri-linux-x64": "npm:@gaossr/kiri@9.9.9-linux-x64",
    "@gaossr/kiri-win32-x64": "npm:@gaossr/kiri@9.9.9-win32-x64",
  });
  assert.ok(!listTarball(rootTarball).includes("package/vendor/.gitkeep"));

  const armPackage = extractPackageJson(armTarball, tempRoot);
  assert.equal(armPackage.name, "@gaossr/kiri");
  assert.equal(armPackage.version, "9.9.9-darwin-arm64");
  assert.deepEqual(armPackage.os, ["darwin"]);
  assert.deepEqual(armPackage.cpu, ["arm64"]);
  assert.equal(armPackage.repository.url, "git+https://github.com/GaoSSR/kiri.git");
  assert.ok(
    listTarball(armTarball).includes("package/vendor/darwin-arm64/ports")
  );

  const x64Package = extractPackageJson(x64Tarball, tempRoot);
  assert.equal(x64Package.name, "@gaossr/kiri");
  assert.equal(x64Package.version, "9.9.9-darwin-x64");
  assert.deepEqual(x64Package.os, ["darwin"]);
  assert.deepEqual(x64Package.cpu, ["x64"]);
  assert.ok(listTarball(x64Tarball).includes("package/vendor/darwin-x64/ports"));

  const linuxPackage = extractPackageJson(linuxTarball, tempRoot);
  assert.equal(linuxPackage.version, "9.9.9-linux-x64");
  assert.deepEqual(linuxPackage.os, ["linux"]);
  assert.deepEqual(linuxPackage.cpu, ["x64"]);
  assert.ok(listTarball(linuxTarball).includes("package/vendor/linux-x64/ports"));

  const windowsPackage = extractPackageJson(windowsTarball, tempRoot);
  assert.equal(windowsPackage.version, "9.9.9-win32-x64");
  assert.deepEqual(windowsPackage.os, ["win32"]);
  assert.deepEqual(windowsPackage.cpu, ["x64"]);
  assert.ok(
    listTarball(windowsTarball).includes("package/vendor/win32-x64/ports.exe")
  );
});
