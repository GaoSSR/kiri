"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const PLATFORM_PACKAGES = {
  "darwin-arm64": "@gaossr/kiri-darwin-arm64",
  "darwin-x64": "@gaossr/kiri-darwin-x64",
};

function platformKey(runtime = process) {
  const platform = runtime.platform;
  const arch = runtime.arch;

  if (platform === "darwin" && arch === "arm64") {
    return "darwin-arm64";
  }
  if (platform === "darwin" && arch === "x64") {
    return "darwin-x64";
  }
  if (platform === "linux" && arch === "x64") {
    return "linux-x64";
  }
  if (platform === "win32" && arch === "x64") {
    return "win32-x64";
  }

  return `${platform}-${arch}`;
}

function binaryName(command) {
  return process.platform === "win32" ? `${command}.exe` : command;
}

function platformPackageName(key) {
  return PLATFORM_PACKAGES[key];
}

function binaryPath(command, options = {}) {
  const packageRoot = options.packageRoot || path.join(__dirname, "..");
  const runtime = options.runtime || process;
  return path.join(packageRoot, "vendor", platformKey(runtime), binaryName(command));
}

function platformPackageBinaryPath(packageRoot, key, command) {
  return path.join(packageRoot, "vendor", key, binaryName(command));
}

function resolveBinary(command, options = {}) {
  const runtime = options.runtime || process;
  const key = platformKey(runtime);
  const platformPackage = platformPackageName(key);

  if (!platformPackage) {
    return null;
  }

  const requireFn = options.requireFn || require;
  try {
    const packageJsonPath = requireFn.resolve(`${platformPackage}/package.json`);
    const resolved = platformPackageBinaryPath(
      path.dirname(packageJsonPath),
      key,
      command
    );
    if (fs.existsSync(resolved)) {
      return resolved;
    }
  } catch {
    // Fall back to local vendor for staged-package verification.
  }

  const localBinary = binaryPath(command, options);
  if (fs.existsSync(localBinary)) {
    return localBinary;
  }

  return null;
}

function runBinary(command) {
  const key = platformKey();
  const resolved = resolveBinary(command);

  if (!platformPackageName(key)) {
    console.error(
      [
        `Kiri does not have a bundled binary for ${key}.`,
        "macOS is supported first; Linux and Windows packages will be added after their collectors ship.",
        "This npm package does not compile Rust locally and does not require Cargo.",
      ].join("\n")
    );
    process.exit(1);
  }

  if (!resolved || !fs.existsSync(resolved)) {
    console.error(
      [
        `Kiri npm package artifacts are missing for ${key}.`,
        "Reinstall Kiri with: npm install -g @gaossr/kiri@latest",
        "The npm package uses precompiled release binaries and does not compile Rust locally.",
        "This npm package does not compile Rust locally and does not require Cargo.",
      ].join("\n")
    );
    process.exit(1);
  }

  const result = spawnSync(resolved, process.argv.slice(2), {
    stdio: "inherit",
  });

  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }

  process.exit(result.status ?? 1);
}

module.exports = {
  binaryPath,
  platformPackageName,
  platformKey,
  resolveBinary,
  runBinary,
};
