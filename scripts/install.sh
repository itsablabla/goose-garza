#!/usr/bin/env bash
set -euo pipefail

GOOSE_VERSION="${GOOSE_VERSION:-1.31.0}"
GOOSE_SERVER_URL="${GOOSE_SERVER_URL:-https://goosed.garzaos.cloud}"
GOOSE_SERVER_SECRET="${GOOSE_SERVER_SECRET:-garza-goosed-secret-2c0a4f7b9e3d81a4c6b2e8f5}"

RELEASE_BASE="https://github.com/block/goose/releases/download/v${GOOSE_VERSION}"

OS="$(uname -s)"
ARCH="$(uname -m)"

log() { echo -e "\033[36m==>\033[0m $*"; }
err() { echo -e "\033[31m!! \033[0m $*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || err "Required tool '$1' not installed"; }

write_settings() {
  local dir="$1"
  mkdir -p "$dir"
  local file="$dir/settings.json"
  local new_json
  new_json=$(cat <<EOF
{
  "externalGoosed": {
    "enabled": true,
    "url": "$GOOSE_SERVER_URL",
    "secret": "$GOOSE_SERVER_SECRET"
  }
}
EOF
)
  if [ -f "$file" ]; then
    if command -v jq >/dev/null 2>&1; then
      jq --arg url "$GOOSE_SERVER_URL" --arg secret "$GOOSE_SERVER_SECRET" \
        '.externalGoosed = {enabled: true, url: $url, secret: $secret}' \
        "$file" > "$file.tmp" && mv "$file.tmp" "$file"
    else
      cp "$file" "$file.bak"
      echo "$new_json" > "$file"
      log "jq missing; overwrote settings.json (backup at $file.bak)"
    fi
  else
    echo "$new_json" > "$file"
  fi
  log "Wrote $file"
}

install_macos() {
  need curl; need unzip
  local url zip app_dir
  if [ "$ARCH" = "arm64" ]; then
    url="$RELEASE_BASE/Goose.zip"
  else
    url="$RELEASE_BASE/Goose_intel_mac.zip"
  fi
  zip="/tmp/GarzaGoose-$$.zip"
  log "Downloading Goose $GOOSE_VERSION for macOS $ARCH..."
  curl -fL# -o "$zip" "$url"
  log "Installing to /Applications (sudo may prompt)..."
  rm -rf "/Applications/Goose.app"
  unzip -q "$zip" -d /Applications/
  rm -f "$zip"
  xattr -dr com.apple.quarantine "/Applications/Goose.app" 2>/dev/null || true
  write_settings "$HOME/Library/Application Support/Goose"
  log "Done! Launching Goose..."
  open -a "/Applications/Goose.app"
}

install_linux() {
  need curl
  local tmp dest
  if command -v dpkg >/dev/null 2>&1; then
    tmp="/tmp/goose-$$.deb"
    log "Downloading .deb..."
    curl -fL# -o "$tmp" "$RELEASE_BASE/goose_${GOOSE_VERSION}_amd64.deb"
    log "Installing with sudo dpkg..."
    sudo dpkg -i "$tmp" || sudo apt-get -f install -y
    rm -f "$tmp"
  elif command -v rpm >/dev/null 2>&1; then
    tmp="/tmp/goose-$$.rpm"
    log "Downloading .rpm..."
    curl -fL# -o "$tmp" "$RELEASE_BASE/Goose-${GOOSE_VERSION}-1.x86_64.rpm"
    log "Installing..."
    sudo rpm -i --replacepkgs "$tmp"
    rm -f "$tmp"
  else
    err "Unsupported Linux distro (no dpkg or rpm). Download the AppImage/flatpak manually."
  fi
  write_settings "$HOME/.config/Goose"
  log "Done! Launch 'goose' from your app menu."
}

case "$OS" in
  Darwin) install_macos ;;
  Linux)  install_linux ;;
  *) err "Unsupported OS: $OS (use install.ps1 on Windows)" ;;
esac

log "Garza Goose is installed and pre-configured:"
log "  Server URL: $GOOSE_SERVER_URL"
log "  Secret Key: (hidden)"
log "On launch, new chats will use the remote goosed backend automatically."
