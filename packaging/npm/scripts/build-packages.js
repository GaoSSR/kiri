#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const PACKAGE_ROOT = path.resolve(__dirname, "..");
const PACKAGE_JSON = path.join(PACKAGE_ROOT, "package.json");

const PLATFORMS = [
  {
    key: "darwin-arm64",
    target: "aarch64-apple-darwin",
    alias: "@gaossr/kiri-darwin-arm64",
    os: "darwin",
    cpu: "arm64",
  },
  {
    key: "darwin-x64",
    target: "x86_64-apple-darwin",
    alias: "@gaossr/kiri-darwin-x64",
    os: "darwin",
    cpu: "x64",
  },
];

function parseArgs(argv) {
  const args = {};
  for (let index = 2; index < argv.length; index += 1) {
    const name = argv[index];
    const value = argv[index + 1];
    if (!name.startsWith("--") || value === undefined || value.startsWith("--")) {
      throw new Error(`missing value for ${name}`);
    }
    args[name.slice(2)] = value;
    index += 1;
  }

  for (const required of ["version", "release-dir", "output-dir"]) {
    if (!args[required]) {
      throw new Error(`missing --${required}`);
    }
  }

  return {
    version: args.version,
    releaseDir: path.resolve(args["release-dir"]),
    outputDir: path.resolve(args["output-dir"]),
  };
}

function readPackageJson() {
  return JSON.parse(fs.readFileSync(PACKAGE_JSON, "utf8"));
}

function copyIfExists(source, destination) {
  if (fs.existsSync(source)) {
    fs.copyFileSync(source, destination);
  }
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function prepareEmptyDir(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

function stageRootPackage(version) {
  const stagingDir = prepareEmptyDir("kiri-npm-root-");
  const sourcePackage = readPackageJson();
  const packageJson = {
    ...sourcePackage,
    version,
    files: ["LICENSE", "README.md", "bin/", "lib/"],
    optionalDependencies: Object.fromEntries(
      PLATFORMS.map((platform) => [
        platform.alias,
        `npm:${sourcePackage.name}@${version}-${platform.key}`,
      ])
    ),
  };
  delete packageJson.private;
  delete packageJson.scripts;

  fs.mkdirSync(path.join(stagingDir, "bin"), { recursive: true });
  fs.mkdirSync(path.join(stagingDir, "lib"), { recursive: true });
  fs.copyFileSync(path.join(PACKAGE_ROOT, "bin", "ports.js"), path.join(stagingDir, "bin", "ports.js"));
  fs.copyFileSync(
    path.join(PACKAGE_ROOT, "lib", "resolve-binary.js"),
    path.join(stagingDir, "lib", "resolve-binary.js")
  );
  copyIfExists(path.join(PACKAGE_ROOT, "LICENSE"), path.join(stagingDir, "LICENSE"));
  copyIfExists(path.join(PACKAGE_ROOT, "README.md"), path.join(stagingDir, "README.md"));
  writeJson(path.join(stagingDir, "package.json"), packageJson);
  return stagingDir;
}

function stagePlatformPackage(version, platform, releaseDir) {
  const archivePath = path.join(releaseDir, `kiri-${platform.target}.tar.gz`);
  if (!fs.existsSync(archivePath)) {
    throw new Error(`missing release archive: ${archivePath}`);
  }

  const stagingDir = prepareEmptyDir(`kiri-npm-${platform.key}-`);
  const extractedDir = prepareEmptyDir(`kiri-release-${platform.key}-`);
  execFileSync("tar", ["-xzf", archivePath, "-C", extractedDir]);

  const sourceBinary = path.join(extractedDir, "ports");
  if (!fs.existsSync(sourceBinary)) {
    throw new Error(`release archive did not contain ports: ${archivePath}`);
  }

  const vendorDir = path.join(stagingDir, "vendor", platform.key);
  fs.mkdirSync(vendorDir, { recursive: true });
  const binaryDest = path.join(vendorDir, "ports");
  fs.copyFileSync(sourceBinary, binaryDest);
  fs.chmodSync(binaryDest, 0o755);

  const sourcePackage = readPackageJson();
  const packageJson = {
    name: sourcePackage.name,
    version: `${version}-${platform.key}`,
    description: `${sourcePackage.description} (${platform.key})`,
    license: sourcePackage.license,
    os: [platform.os],
    cpu: [platform.cpu],
    files: ["LICENSE", "README.md", "vendor/"],
    engines: sourcePackage.engines,
  };

  copyIfExists(path.join(PACKAGE_ROOT, "LICENSE"), path.join(stagingDir, "LICENSE"));
  copyIfExists(path.join(PACKAGE_ROOT, "README.md"), path.join(stagingDir, "README.md"));
  writeJson(path.join(stagingDir, "package.json"), packageJson);
  return stagingDir;
}

function npmPack(stagingDir, outputDir, outputName) {
  fs.mkdirSync(outputDir, { recursive: true });
  const packDir = prepareEmptyDir("kiri-npm-pack-");
  const output = execFileSync(
    "npm",
    ["pack", "--json", "--pack-destination", packDir],
    {
      cwd: stagingDir,
      encoding: "utf8",
      env: {
        ...process.env,
        NPM_CONFIG_CACHE: path.join(packDir, "cache"),
        NPM_CONFIG_LOGS_DIR: path.join(packDir, "logs"),
      },
    }
  );
  const packed = JSON.parse(output);
  if (!Array.isArray(packed) || !packed[0] || !packed[0].filename) {
    throw new Error("npm pack did not report a tarball filename");
  }

  const generatedPath = path.join(packDir, packed[0].filename);
  const finalPath = path.join(outputDir, outputName);
  fs.renameSync(generatedPath, finalPath);
  return finalPath;
}

function main() {
  const args = parseArgs(process.argv);
  const outputs = [];

  const rootStaging = stageRootPackage(args.version);
  outputs.push(npmPack(rootStaging, args.outputDir, `kiri-npm-${args.version}.tgz`));

  for (const platform of PLATFORMS) {
    const staging = stagePlatformPackage(args.version, platform, args.releaseDir);
    outputs.push(
      npmPack(staging, args.outputDir, `kiri-npm-${platform.key}-${args.version}.tgz`)
    );
  }

  for (const output of outputs) {
    console.log(output);
  }
}

main();
