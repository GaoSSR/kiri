"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");

const {
  extractChangelogSection,
  normalizeVersion,
  renderReleaseNotes,
} = require("./generate-release-notes");

const changelog = `# Changelog

## 0.1.7 - 2026-05-23

### Added

- Added Linux x64 support.

### Changed

- Updated install docs.

## 0.1.6 - 2026-05-23

### Added

- Added npm packaging.
`;

test("normalizes tags and plain versions consistently", () => {
  assert.equal(normalizeVersion("v0.1.7"), "0.1.7");
  assert.equal(normalizeVersion("0.1.7"), "0.1.7");
});

test("extracts only the requested changelog section", () => {
  const section = extractChangelogSection(changelog, "v0.1.7");

  assert.match(section, /Added Linux x64 support/);
  assert.match(section, /Updated install docs/);
  assert.doesNotMatch(section, /Added npm packaging/);
});

test("fails when the requested version is missing", () => {
  assert.throws(
    () => extractChangelogSection(changelog, "0.2.0"),
    /does not contain section/
  );
});

test("renders install commands, assets, and compare link", () => {
  const notes = renderReleaseNotes({
    version: "v0.1.7",
    changelog,
    repo: "GaoSSR/kiri",
    previousTag: "v0.1.6",
  });

  assert.match(notes, /^# Kiri 0\.1\.7/m);
  assert.match(notes, /^## Highlights/m);
  assert.match(notes, /npm install -g @gaossr\/kiri/);
  assert.match(notes, /kiri-x86_64-unknown-linux-musl\.tar\.gz/);
  assert.match(notes, /kiri-x86_64-pc-windows-msvc\.zip/);
  assert.match(notes, /kiri-npm-win32-x64-0\.1\.7\.tgz/);
  assert.match(notes, /compare\/v0\.1\.6\.\.\.v0\.1\.7/);
});
