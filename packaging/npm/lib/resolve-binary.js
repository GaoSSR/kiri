"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const SUPPORTED_PACKAGE_PLATFORMS = new Set(["darwin-arm64", "darwin-x64"]);

function platformKey() {
  const platform = process.platform;
  const arch = process.arch;

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

function binaryPath(command) {
  return path.join(__dirname, "..", "vendor", platformKey(), binaryName(command));
}

function runBinary(command) {
  const resolved = binaryPath(command);
  const key = platformKey();

  if (!SUPPORTED_PACKAGE_PLATFORMS.has(key)) {
    console.error(
      [
        `Kiri does not have a bundled binary for ${key}.`,
        "macOS is supported first; Linux and Windows packages will be added after their collectors ship.",
        "This npm package does not compile Rust locally and does not require Cargo.",
      ].join("\n")
    );
    process.exit(1);
  }

  if (!fs.existsSync(resolved)) {
    console.error(
      [
        `Kiri npm package artifacts are not bundled yet for ${key}.`,
        "This package scaffold is prepared for future release packaging.",
        "Install from npm only after an official Kiri release publishes precompiled binaries.",
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
  platformKey,
  runBinary,
};
