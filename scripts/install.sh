#!/usr/bin/env bash
set -euo pipefail

REPO="${ATTESTACK_INSTALL_REPO:-kiket-dev/attestack}"
VERSION="${ATTESTACK_VERSION:-latest}"
INSTALL_DIR="${ATTESTACK_INSTALL_DIR:-$HOME/.local/bin}"

detect_platform() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "$os" in
    linux)
      case "$arch" in
        x86_64) echo "attestack-linux-x86_64" ;;
        aarch64|arm64) echo "attestack-linux-aarch64" ;;
        *) echo "unsupported linux architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    darwin)
      case "$arch" in
        x86_64) echo "attestack-macos-x86_64" ;;
        arm64) echo "attestack-macos-aarch64" ;;
        *) echo "unsupported macOS architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    mingw*|msys*|cygwin*|windows*)
      echo "attestack-windows-x86_64"
      ;;
    *)
      echo "unsupported OS: $os" >&2
      exit 1
      ;;
  esac
}

resolve_version() {
  if [[ "$VERSION" != "latest" ]]; then
    echo "$VERSION"
    return
  fi
  if command -v gh >/dev/null 2>&1; then
    gh release view --repo "$REPO" --json tagName -q .tagName 2>/dev/null && return
  fi
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -1
}

download_release() {
  local platform="$1"
  local version="$2"
  local archive="$3"
  local dest="$4"

  local url="https://github.com/${REPO}/releases/download/${version}/${archive}"
  if curl -fsSL "$url" -o "$dest"; then
    return 0
  fi

  if command -v gh >/dev/null 2>&1; then
    echo "Anonymous download failed; retrying with gh (required for private repos)." >&2
    gh release download "$version" --repo "$REPO" --pattern "$archive" --dir "$(dirname "$dest")"
    if [[ ! -f "$dest" ]]; then
      echo "gh release download did not produce ${dest}." >&2
      return 1
    fi
    return 0
  fi

  echo "Failed to download ${archive} from ${url}." >&2
  echo "For private repositories, install GitHub CLI and run: gh auth login" >&2
  return 1
}

download_checksum() {
  local platform="$1"
  local version="$2"
  local archive="$3"
  local dest="$4"

  local checksum_url="https://github.com/${REPO}/releases/download/${version}/${archive}.sha256"
  if curl -fsSL "$checksum_url" -o "$dest"; then
    return 0
  fi

  if command -v gh >/dev/null 2>&1; then
    gh release download "$version" --repo "$REPO" --pattern "${archive}.sha256" --dir "$(dirname "$dest")"
    if [[ ! -f "$dest" ]]; then
      echo "gh release download did not produce ${dest}." >&2
      return 1
    fi
    return 0
  fi

  echo "Failed to download checksum for ${archive}." >&2
  return 1
}

install_binaries() {
  local extract_dir="$1"
  install -m 0755 "${extract_dir}/attestack" "${INSTALL_DIR}/attestack"
  if [[ -f "${extract_dir}/attestack-mcp" ]]; then
    install -m 0755 "${extract_dir}/attestack-mcp" "${INSTALL_DIR}/attestack-mcp"
  fi
}

main() {
  local platform archive url tmpdir
  platform="$(detect_platform)"
  VERSION="$(resolve_version)"
  if [[ -z "$VERSION" ]]; then
    echo "Could not resolve release version for ${REPO}." >&2
    exit 1
  fi

  archive="${platform}.tar.gz"
  mkdir -p "$INSTALL_DIR"
  tmpdir="$(mktemp -d)"
  trap "rm -rf '${tmpdir}'" EXIT

  echo "Installing Attestack ${VERSION} (${platform}) to ${INSTALL_DIR}"
  download_release "$platform" "$VERSION" "$archive" "${tmpdir}/${archive}"
  download_checksum "$platform" "$VERSION" "$archive" "${tmpdir}/${archive}.sha256"
  (cd "$tmpdir" && sha256sum -c "${archive}.sha256")
  tar -xzf "${tmpdir}/${archive}" -C "$tmpdir"
  install_binaries "$tmpdir"

  if ! command -v attestack >/dev/null 2>&1; then
    echo "Add ${INSTALL_DIR} to your PATH if needed."
  fi

  "${INSTALL_DIR}/attestack" --help >/dev/null
  echo "Attestack installed successfully."
}

main "$@"
