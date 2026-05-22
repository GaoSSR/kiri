"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

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

  if (!fs.existsSync(resolved)) {
    console.error(
      [
        `Kiri npm package artifacts are not bundled yet for ${platformKey()}.`,
        "This package scaffold is prepared for future release packaging.",
        "Install from npm only after an official Kiri release publishes precompiled binaries.",
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
