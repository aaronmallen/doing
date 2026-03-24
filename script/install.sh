#!/bin/sh
# Install script for doing — https://github.com/aaronmallen/doing
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/aaronmallen/doing/main/script/install.sh | sh
#
# Environment variables:
#   DOING_VERSION       — version to install (default: latest)
#   DOING_INSTALL_PATH  — install directory (default: ~/.local/bin)

set -eu

REPO="aaronmallen/doing"
INSTALL_DIR="${DOING_INSTALL_PATH:-${HOME}/.local/bin}"

main() {
  os="$(detect_os)"
  arch="$(detect_arch)"
  target="$(build_target "$os" "$arch")"
  version="$(resolve_version)"

  archive="doing-v${version}-${target}.tar.gz"
  checksum_file="${archive}.sha256"
  base_url="https://github.com/${REPO}/releases/download/${version}"

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  log "Installing doing v${version} (${target})"

  download "${base_url}/${archive}" "${tmpdir}/${archive}"
  download "${base_url}/${checksum_file}" "${tmpdir}/${checksum_file}"
  verify_checksum "${tmpdir}/${archive}" "${tmpdir}/${checksum_file}"

  tar xzf "${tmpdir}/${archive}" -C "${tmpdir}"

  mkdir -p "${INSTALL_DIR}"
  mv "${tmpdir}/doing" "${INSTALL_DIR}/doing"
  chmod +x "${INSTALL_DIR}/doing"

  log "Installed doing to ${INSTALL_DIR}/doing"
  check_path
}

detect_os() {
  case "$(uname -s)" in
    Darwin) echo "macos" ;;
    Linux)  echo "linux" ;;
    *)      err "unsupported OS: $(uname -s)" ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    x86_64)         echo "x86_64" ;;
    aarch64 | arm64) echo "aarch64" ;;
    *)              err "unsupported architecture: $(uname -m)" ;;
  esac
}

build_target() {
  os="$1"
  arch="$2"

  case "$os" in
    macos) echo "${arch}-apple-darwin" ;;
    linux) echo "${arch}-unknown-linux-musl" ;;
  esac
}

resolve_version() {
  if [ -n "${DOING_VERSION:-}" ]; then
    echo "${DOING_VERSION#v}"
    return
  fi

  url="https://api.github.com/repos/${REPO}/releases/latest"
  if has curl; then
    version="$(curl -fsSL "$url" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')"
  elif has wget; then
    version="$(wget -qO- "$url" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')"
  else
    err "curl or wget is required"
  fi

  if [ -z "$version" ]; then
    err "could not determine latest version"
  fi

  echo "${version#v}"
}

download() {
  url="$1"
  dest="$2"

  if has curl; then
    curl -fsSL -o "$dest" "$url"
  elif has wget; then
    wget -qO "$dest" "$url"
  else
    err "curl or wget is required"
  fi
}

verify_checksum() {
  file="$1"
  checksum_file="$2"

  expected="$(awk '{print $1}' "$checksum_file")"

  if has sha256sum; then
    actual="$(sha256sum "$file" | awk '{print $1}')"
  elif has shasum; then
    actual="$(shasum -a 256 "$file" | awk '{print $1}')"
  else
    log "warning: could not verify checksum (no sha256sum or shasum found)"
    return
  fi

  if [ "$actual" != "$expected" ]; then
    err "checksum mismatch: expected ${expected}, got ${actual}"
  fi

  log "Checksum verified"
}

check_path() {
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
      log ""
      log "Add ${INSTALL_DIR} to your PATH:"
      log "  export PATH=\"${INSTALL_DIR}:\$PATH\""
      ;;
  esac
}

has() {
  command -v "$1" >/dev/null 2>&1
}

log() {
  printf '%s\n' "$1"
}

err() {
  log "error: $1" >&2
  exit 1
}

main
