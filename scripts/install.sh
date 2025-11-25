#!/usr/bin/env bash
# Luma Unix/macOS install script
# Mirrors functionality of scripts/install.ps1 for Windows.
# Usage (latest): curl -fsSL https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.sh | sh

set -euo pipefail

C_RESET="\033[0m"
C_GREEN="\033[1;32m"
C_YELLOW="\033[1;33m"
C_RED="\033[1;31m"

VERSION="latest"
NO_PATH_UPDATE=0
INSTALL_DIR="" # default resolved later (~/.luma)
FORCE=0
QUIET=0

print_help() {
  cat <<'EOF'
Luma Unix Installer

Flags:
  --version <v|semver>   Install specific version (e.g. 0.1.3 or v0.1.3). Default: latest
  --dir <path>           Custom installation root (default: $HOME/.luma)
  --no-path-update       Do not modify shell profile PATH
  --force                Overwrite any existing luma binary
  --quiet                Less output
  --help                 Show this help

Environment:
  LUMA_INSTALL           Overrides installation root if set

Examples:
  # Latest
  curl -fsSL https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.sh | sh

  # Specific version
  curl -fsSL https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.sh | sh -s -- --version 0.1.3

  # Custom directory without PATH modification
  curl -fsSL https://raw.githubusercontent.com/tayadev/luma/refs/heads/main/scripts/install.sh | bash -s -- --dir /opt/luma --no-path-update
EOF
}

log() { if [ "$QUIET" -eq 0 ]; then printf "%s\n" "$*"; fi; }
warn() { printf "%b[WARN]%b %s\n" "$C_YELLOW" "$C_RESET" "$*" >&2; }
err()  { printf "%b[ERROR]%b %s\n" "$C_RED" "$C_RESET" "$*" >&2; }

# Parse args
while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2;;
    --dir) INSTALL_DIR="$2"; shift 2;;
    --no-path-update) NO_PATH_UPDATE=1; shift;;
    --force) FORCE=1; shift;;
    --quiet) QUIET=1; shift;;
    --help|-h) print_help; exit 0;;
    *) err "Unknown argument: $1"; print_help; exit 1;;
  esac
done

# Normalize version
if [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  VERSION="v$VERSION"
fi

OS_UNAME=$(uname -s)
ARCH_UNAME=$(uname -m)

case "$OS_UNAME" in
  Linux)   OS_TAG="linux";;
  Darwin)  OS_TAG="macos";;
  *) err "Unsupported OS: $OS_UNAME"; exit 1;;
esac

case "$ARCH_UNAME" in
  x86_64|amd64) ARCH_TAG="x64";;
  arm64|aarch64) ARCH_TAG="aarch64";;
  *) err "Unsupported architecture: $ARCH_UNAME"; exit 1;;
esac

TARGET="${OS_TAG}-${ARCH_TAG}"

# Determine install root
if [ -n "$LUMA_INSTALL" ]; then
  ROOT="$LUMA_INSTALL"
else
  if [ -n "$INSTALL_DIR" ]; then
    ROOT="$INSTALL_DIR"
  else
    ROOT="$HOME/.luma"
  fi
fi
BIN_DIR="$ROOT/bin"
mkdir -p "$BIN_DIR"

ZIP_NAME="$TARGET.zip"
ZIP_PATH="$BIN_DIR/$ZIP_NAME"

BASE_URL="https://github.com/tayadev/luma/releases"
if [ "$VERSION" = "latest" ]; then
  DOWNLOAD_URL="$BASE_URL/latest/download/$ZIP_NAME"
else
  DOWNLOAD_URL="$BASE_URL/download/$VERSION/$ZIP_NAME"
fi

# Pre-flight checks
if command -v luma >/dev/null 2>&1; then
  EXISTING_PATH=$(command -v luma)
  if [ "$EXISTING_PATH" != "$BIN_DIR/luma" ]; then
    warn "Another luma is already in PATH at $EXISTING_PATH"
  fi
fi

if [ -f "$BIN_DIR/luma" ] && [ "$FORCE" -ne 1 ]; then
  warn "luma already installed at $BIN_DIR/luma (use --force to overwrite)"
fi

# Download
log "Downloading $DOWNLOAD_URL"
if command -v curl >/dev/null 2>&1; then
  if ! curl -fsSL "$DOWNLOAD_URL" -o "$ZIP_PATH"; then
    err "Download failed via curl"; exit 1; fi
elif command -v wget >/dev/null 2>&1; then
  if ! wget -q "$DOWNLOAD_URL" -O "$ZIP_PATH"; then
    err "Download failed via wget"; exit 1; fi
else
  err "Need curl or wget to download."; exit 1
fi

if [ ! -s "$ZIP_PATH" ]; then
  err "Downloaded file is empty: $ZIP_PATH"; exit 1
fi

# Extract (requires unzip)
if ! command -v unzip >/dev/null 2>&1; then
  err "'unzip' is required to extract $ZIP_NAME"; exit 1
fi

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
unzip -q "$ZIP_PATH" -d "$TMP_DIR"
rm -f "$ZIP_PATH"

if [ ! -f "$TMP_DIR/$TARGET/luma" ]; then
  err "Expected binary '$TMP_DIR/$TARGET/luma' not found. Artifact layout changed?"; exit 1
fi

mv "$TMP_DIR/$TARGET/luma" "$BIN_DIR/luma"
chmod +x "$BIN_DIR/luma"
rm -rf "$TMP_DIR/$TARGET"

# Verify
if ! "$BIN_DIR/luma" --version >/dev/null 2>&1; then
  err "Installed luma failed to execute."; exit 1
fi

INSTALLED_VERSION=$("$BIN_DIR/luma" --version || true)
printf "%bLuma %s was installed successfully!%b\n" "$C_GREEN" "$INSTALLED_VERSION" "$C_RESET"
log "Binary location: $BIN_DIR/luma"

# PATH handling
if [ "$NO_PATH_UPDATE" -eq 1 ]; then
  log "Skipping PATH modification (--no-path-update)"
else
  case ":$PATH:" in
    *":$BIN_DIR:"*) log "Install directory already in PATH";;
    *)
      PROFILE_FILES=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile")
      ADDED=0
      for f in "${PROFILE_FILES[@]}"; do
        if [ -f "$f" ]; then
          if ! grep -q "luma/bin" "$f"; then
            printf '\n# Added by Luma installer\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$f"
            ADDED=1
            log "Appended PATH export to $f"
            break
          fi
        fi
      done
      if [ $ADDED -eq 0 ]; then
        warn "Could not find a profile to update PATH automatically. Add this line manually:"
        printf 'export PATH="%s:$PATH"\n' "$BIN_DIR"
      fi
      ;;
  esac
fi

log "Restart your terminal or run: export PATH=\"$BIN_DIR:$PATH\""
log "Then type: luma --help"

exit 0
