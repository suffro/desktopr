# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

if [[ "$OS" == "Darwin" ]]; then
  # macOS
  if [[ "$ARCH" == "arm64" ]]; then
    export TAURI_ENV_TARGET_TRIPLE="aarch64-apple-darwin"
  else
    export TAURI_ENV_TARGET_TRIPLE="x86_64-apple-darwin"
  fi
elif [[ "$OS" == "Linux" ]]; then
  # Linux x86_64
  export TAURI_ENV_TARGET_TRIPLE="x86_64-unknown-linux-gnu"
elif [[ "$OS" =~ MINGW|MSYS|CYGWIN ]]; then
  # Windows (Git Bash)
  export TAURI_ENV_TARGET_TRIPLE="x86_64-pc-windows-msvc"
else
  echo "Unsupported OS: $OS"
  exit 1
fi

echo "TAURI_ENV_TARGET_TRIPLE set to: $TAURI_ENV_TARGET_TRIPLE"

echo "Preparing companion binary for this target..."

TARGET_TRIPLE="${TAURI_ENV_TARGET_TRIPLE:-}"

if [ -z "$TARGET_TRIPLE" ]; then
  echo "ERROR: TAURI_ENV_TARGET_TRIPLE is missing!"
  exit 1
fi

DEST_DIR="src-tauri/resources/companion"
mkdir -p "$DEST_DIR"

case "$TARGET_TRIPLE" in
  "aarch64-apple-darwin")
    SRC_BIN="src-tauri/binaries/bubbledesk-companion-aarch64-apple-darwin"
    DEST_BIN="$DEST_DIR/bubbledesk-companion"
    ;;
  "x86_64-apple-darwin")
    SRC_BIN="src-tauri/binaries/bubbledesk-companion-x86_64-apple-darwin"
    DEST_BIN="$DEST_DIR/bubbledesk-companion"
    ;;
  "x86_64-pc-windows-msvc")
    SRC_BIN="src-tauri/binaries/bubbledesk-companion-x86_64-pc-windows-msvc.exe"
    DEST_BIN="$DEST_DIR/bubbledesk-companion.exe"
    ;;
  "x86_64-unknown-linux-gnu")
    SRC_BIN="src-tauri/binaries/bubbledesk-companion-x86_64-unknown-linux-gnu"
    DEST_BIN="$DEST_DIR/bubbledesk-companion"
    ;;
  *)
    echo "ERROR: Unsupported TAURI_ENV_TARGET_TRIPLE: $TARGET_TRIPLE"
    exit 1
    ;;
esac

echo "Copying companion binary for target: $TARGET_TRIPLE"
echo "  from: $SRC_BIN"
echo "  to:   $DEST_BIN"

cp "$SRC_BIN" "$DEST_BIN"
chmod +x "$DEST_BIN"

echo "Companion binary ready."