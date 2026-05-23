#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const RELEASE_ASSETS = [
  {
    name: "kiri-aarch64-apple-darwin.tar.gz",
    platform: "macOS Apple Silicon",
  },
  {
    name: "kiri-x86_64-apple-darwin.tar.gz",
    platform: "macOS Intel",
  },
  {
    name: "kiri-x86_64-unknown-linux-musl.tar.gz",
    platform: "Linux x64",
  },
  {
    name: "kiri-x86_64-pc-windows-msvc.zip",
    platform: "Windows x64",
  },
  {
    name: "checksums.txt",
    platform: "SHA-256 checksums",
  },
];

const NPM_ASSETS = [
  "kiri-npm-<version>.tgz",
  "kiri-npm-darwin-arm64-<version>.tgz",
  "kiri-npm-darwin-x64-<version>.tgz",
  "kiri-npm-linux-x64-<version>.tgz",
  "kiri-npm-win32-x64-<version>.tgz",
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

  for (const required of ["version", "changelog", "output", "repo"]) {
    if (!args[required]) {
      throw new Error(`missing --${required}`);
    }
  }

  return {
    version: normalizeVersion(args.version),
    changelogPath: path.resolve(args.changelog),
    outputPath: path.resolve(args.output),
    repo: args.repo,
    previousTag: args["previous-tag"],
  };
}

function normalizeVersion(version) {
  return version.startsWith("v") ? version.slice(1) : version;
}

function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function extractChangelogSection(markdown, version) {
  const lines = markdown.split(/\r?\n/);
  const headingPattern = new RegExp(
    `^##\\s+${escapeRegex(normalizeVersion(version))}(?:\\s+-.*)?\\s*$`
  );
  const nextVersionPattern = /^##\s+\S/;

  let start = -1;
  for (let index = 0; index < lines.length; index += 1) {
    if (headingPattern.test(lines[index])) {
      start = index + 1;
      break;
    }
  }

  if (start === -1) {
    throw new Error(`CHANGELOG.md does not contain section for ${version}`);
  }

  let end = lines.length;
  for (let index = start; index < lines.length; index += 1) {
    if (nextVersionPattern.test(lines[index])) {
      end = index;
      break;
    }
  }

  const section = lines.slice(start, end).join("\n").trim();
  if (!section) {
    throw new Error(`CHANGELOG.md section for ${version} is empty`);
  }

  return section;
}

function renderReleaseNotes(options) {
  const version = normalizeVersion(options.version);
  const changelog = extractChangelogSection(options.changelog, version);
  const tag = `v${version}`;
  const releaseBase = `https://github.com/${options.repo}/releases/download/${tag}`;
  const installScript = `https://raw.githubusercontent.com/${options.repo}/main/scripts`;
  const lines = [];

  lines.push(`# Kiri ${version}`);
  lines.push("");
  lines.push("## Highlights");
  lines.push("");
  lines.push(changelog);
  lines.push("");
  lines.push("## Install");
  lines.push("");
  lines.push("```bash");
  lines.push("npm install -g @gaossr/kiri");
  lines.push("brew install gaossr/tap/kiri");
  lines.push(`curl -fsSL ${installScript}/install.sh | bash`);
  lines.push("```");
  lines.push("");
  lines.push("```powershell");
  lines.push(`irm ${installScript}/install.ps1 | iex`);
  lines.push("```");
  lines.push("");
  lines.push("## Release Assets");
  lines.push("");
  for (const asset of RELEASE_ASSETS) {
    lines.push(`- \`${asset.name}\` - ${asset.platform}`);
  }
  lines.push("");
  lines.push("## npm Assets");
  lines.push("");
  for (const asset of NPM_ASSETS) {
    lines.push(`- \`${asset.replace("<version>", version)}\``);
  }
  lines.push("");
  lines.push("## Checksums");
  lines.push("");
  lines.push(`Download \`checksums.txt\` from ${releaseBase}/checksums.txt.`);

  if (options.previousTag) {
    lines.push("");
    lines.push("## Changelog");
    lines.push("");
    lines.push(
      `Full Changelog: https://github.com/${options.repo}/compare/${options.previousTag}...${tag}`
    );
  }

  return `${lines.join("\n")}\n`;
}

function main() {
  const args = parseArgs(process.argv);
  const changelog = fs.readFileSync(args.changelogPath, "utf8");
  const notes = renderReleaseNotes({
    version: args.version,
    changelog,
    repo: args.repo,
    previousTag: args.previousTag,
  });

  fs.mkdirSync(path.dirname(args.outputPath), { recursive: true });
  fs.writeFileSync(args.outputPath, notes);
}

if (require.main === module) {
  main();
}

module.exports = {
  extractChangelogSection,
  normalizeVersion,
  renderReleaseNotes,
};
