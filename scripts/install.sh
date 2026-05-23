#!/usr/bin/env bash
set -euo pipefail

repo="${KIRI_REPO:-gaossr/kiri}"
version="${KIRI_VERSION:-latest}"
install_dir="${KIRI_INSTALL_DIR:-${HOME}/.local/bin}"

say() {
  printf '%s\n' "$1"
}

fail() {
  say "kiri: $1" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin)
      case "$arch" in
        arm64|aarch64) printf 'aarch64-apple-darwin' ;;
        x86_64|amd64) printf 'x86_64-apple-darwin' ;;
        *) fail "unsupported macOS architecture: $arch" ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64|amd64) printf 'x86_64-unknown-linux-musl' ;;
        *) fail "unsupported Linux architecture: $arch" ;;
      esac
      ;;
    MINGW*|MSYS*|CYGWIN*)
      fail "Use scripts/install.ps1 for Windows x64 installs."
      ;;
    *)
      fail "unsupported platform: $os $arch"
      ;;
  esac
}

download_url() {
  local artifact="$1"
  if [ "$version" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download/%s' "$repo" "$artifact"
  else
    printf 'https://github.com/%s/releases/download/%s/%s' "$repo" "$version" "$artifact"
  fi
}

need curl
need tar
need mktemp

target="$(detect_target)"
artifact="kiri-${target}.tar.gz"
url="$(download_url "$artifact")"
tmp_dir="$(mktemp -d)"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

say "Installing Kiri for ${target}"
say "Downloading ${url}"

curl -fsSL "$url" -o "${tmp_dir}/${artifact}" || fail "failed to download ${artifact}. Has a Kiri release been published?"
tar -xzf "${tmp_dir}/${artifact}" -C "$tmp_dir"

if [ ! -f "${tmp_dir}/ports" ]; then
  fail "release archive did not contain the ports binary"
fi

mkdir -p "$install_dir"
install -m 755 "${tmp_dir}/ports" "${install_dir}/ports"

say "Installed ports to ${install_dir}/ports"
case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *)
    say "Add ${install_dir} to PATH if your shell cannot find ports."
    ;;
esac

say "Next: ports"
