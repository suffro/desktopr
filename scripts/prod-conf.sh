# scripts/prod-conf.sh
set -euo pipefail

# -----------------------------
# Read inputs with safe defaults
# -----------------------------
: "${APP_URL:=}"
: "${APP_VERSION:=0.2.1}"
: "${CARGO_PACKAGE_NAME:=desktopr-wrapper}"
: "${CARGO_PACKAGE_VERSION:=$APP_VERSION}"
: "${APP_IDENTIFIER:=app.desktopr.app}"
: "${UPDATE_ENDPOINT:=}"
: "${TAURI_SIGNING_PUBLIC_KEY:=}"

if [ -z "${ED25519_PUBKEY:-}" ] && [ -n "${TAURI_SIGNING_PUBLIC_KEY:-}" ]; then
  ED25519_PUBKEY="$TAURI_SIGNING_PUBLIC_KEY"
fi

: "${ED25519_PUBKEY:=}"

if { [ -n "$UPDATE_ENDPOINT" ] && [ -z "$ED25519_PUBKEY" ]; } ||
   { [ -z "$UPDATE_ENDPOINT" ] && [ -n "$ED25519_PUBKEY" ]; }; then
  echo "Updater configuration requires both UPDATE_ENDPOINT and ED25519_PUBKEY"
  exit 1
fi

UPDATER_ENABLED=false
if [ -n "$UPDATE_ENDPOINT" ] && [ -n "$ED25519_PUBKEY" ]; then
  if ! printf '%s' "$UPDATE_ENDPOINT" | grep -Eq '^https://[^/]+(/.*)?$'; then
    echo "UPDATE_ENDPOINT must be an absolute HTTPS URL"
    exit 1
  fi
  UPDATER_ENABLED=true
fi
: "${DEEPLINK_SCHEME:=}"
: "${MAIN_WINDOW_TITLE:=Desktopr}"
: "${MAIN_WINDOW_WIDTH:=1200}"
: "${MAIN_WINDOW_HEIGHT:=800}"
: "${MAIN_WINDOW_BG_COLOR:=#ffffff}"
: "${MAIN_WINDOW_URL:=$APP_URL}"
: "${MAIN_WINDOW_RESIZABLE:=true}"
: "${MAIN_WINDOW_OPEN_FULLSCREEN:=false}"
: "${COMPANION_MODE:=false}"
# standalone: bundled fallback page (or APP_URL). bundled: the developer's own
# built web application. companion: bundled Desktopr Companion app.
: "${APP_FRONTEND:=standalone}"
# Repository-relative directory holding the built web application, for bundled.
: "${APP_FRONTEND_DIST:=}"

case "$APP_FRONTEND" in
  standalone) ;;
  bundled)
    if [ -n "$APP_URL" ]; then
      echo "APP_FRONTEND=bundled serves its own files and cannot be combined with APP_URL"
      exit 1
    fi
    if [ -z "$APP_FRONTEND_DIST" ]; then
      echo "APP_FRONTEND=bundled requires APP_FRONTEND_DIST"
      exit 1
    fi
    case "$APP_FRONTEND_DIST" in
      /* | *..*)
        echo "APP_FRONTEND_DIST must be a relative path without '..'"
        exit 1
        ;;
    esac
    if [ ! -d "$APP_FRONTEND_DIST" ]; then
      echo "APP_FRONTEND_DIST is not a directory: $APP_FRONTEND_DIST"
      exit 1
    fi
    # Tauri serves the directory root; without an entry point the window is blank.
    if [ ! -s "$APP_FRONTEND_DIST/index.html" ]; then
      echo "APP_FRONTEND_DIST must contain a non-empty index.html: $APP_FRONTEND_DIST/index.html"
      exit 1
    fi
    ;;
  companion)
    if [ -n "$APP_URL" ]; then
      echo "APP_FRONTEND=companion bundles its own UI and cannot be combined with APP_URL"
      exit 1
    fi
    # The companion loads any web app the user enters in the main window.
    COMPANION_MODE=true
    ;;
  *)
    echo "APP_FRONTEND must be standalone, bundled or companion"
    exit 1
    ;;
esac

cat > src-tauri/window.env << EOF
MAIN_WINDOW_URL=${MAIN_WINDOW_URL}
MAIN_WINDOW_TITLE=${MAIN_WINDOW_TITLE}
MAIN_WINDOW_WIDTH=${MAIN_WINDOW_WIDTH}
MAIN_WINDOW_HEIGHT=${MAIN_WINDOW_HEIGHT}
MAIN_WINDOW_BG_COLOR=${MAIN_WINDOW_BG_COLOR}
MAIN_WINDOW_RESIZABLE=${MAIN_WINDOW_RESIZABLE}
MAIN_WINDOW_VISIBLE=false
MAIN_WINDOW_OPEN_FULLSCREEN=${MAIN_WINDOW_OPEN_FULLSCREEN}
EOF

# -----------------------------
# Resolve APP_URL origin and remote URL patterns
# -----------------------------

APP_URL_ORIGIN=""
APP_URL_SCHEME=""
APP_URL_HOST_WITH_PORT=""
APP_URL_WILDCARD_HOST=""

if [ -n "$APP_URL" ]; then
  APP_URL_ORIGIN="$(printf '%s' "$APP_URL" | sed -E 's#^(https?://[^/]+).*$#\1#')"

  if ! printf '%s' "$APP_URL_ORIGIN" | grep -Eq '^https?://[^/]+$'; then
    echo "Invalid APP_URL: expected an absolute HTTP(S) URL"
    exit 1
  fi

  APP_URL_SCHEME="$(printf '%s' "$APP_URL_ORIGIN" | sed -E 's#^(https?)://.*$#\1#')"
  APP_URL_HOST="$(printf '%s' "$APP_URL_ORIGIN" | sed -E 's#^https?://([^/:]+)(:[0-9]+)?$#\1#')"
  APP_URL_PORT="$(printf '%s' "$APP_URL_ORIGIN" | sed -nE 's#^https?://[^/:]+(:[0-9]+)$#\1#p')"
  APP_URL_HOST_WITH_PORT="${APP_URL_HOST}${APP_URL_PORT}"
  APP_URL_WILDCARD_HOST="*.${APP_URL_HOST}${APP_URL_PORT}"

  echo "Resolved external application origin=$APP_URL_ORIGIN"
else
  case "$APP_FRONTEND" in
    bundled) echo "Application source=bundled files from $APP_FRONTEND_DIST" ;;
    companion) echo "Application source=bundled companion app" ;;
    *) echo "Application source=bundled standalone page" ;;
  esac
fi

if [ "$COMPANION_MODE" = "true" ]; then
  echo "COMPANION_MODE=true (this build accepts IPC from any https origin)"
fi

# -----------------------------
# Always use PROD templates
# -----------------------------

TAURI_TEMPLATE="conf-templates/tauri.conf.template.prod.json"

# -----------------------------
# Generate files from templates
# -----------------------------

echo "1. Generating files from templates"

# remote.json capabilities
cp conf-templates/remote.template.json src-tauri/capabilities/remote.json

if [ "$COMPANION_MODE" = "true" ]; then
  jq '
    .remote = (.remote // {}) |
    .remote.urls = [
      "https://*/*",
      "https://*.*/*",
      "http://localhost/*",
      "http://localhost:*/*",
      "http://127.0.0.1/*",
      "http://127.0.0.1:*/*"
    ]
  ' src-tauri/capabilities/remote.json > src-tauri/capabilities/remote.json.tmp && mv src-tauri/capabilities/remote.json.tmp src-tauri/capabilities/remote.json
elif [ -n "$APP_URL_ORIGIN" ]; then
  jq \
    --arg scheme "$APP_URL_SCHEME" \
    --arg host "$APP_URL_HOST_WITH_PORT" \
    --arg wildcardHost "$APP_URL_WILDCARD_HOST" \
    '
    .remote = (.remote // {}) |
    .remote.urls = [
      ($scheme + "://" + $host + "/*"),
      ($scheme + "://" + $wildcardHost + "/*")
    ]
    ' src-tauri/capabilities/remote.json > src-tauri/capabilities/remote.json.tmp && mv src-tauri/capabilities/remote.json.tmp src-tauri/capabilities/remote.json
fi

echo "  remote.json             -> patched"
echo "  remote.urls:"
jq '.remote.urls // []' src-tauri/capabilities/remote.json

# Cargo.toml
sed -e "s/%%CARGO_PACKAGE_NAME%%/${CARGO_PACKAGE_NAME}/g" \
    -e "s/%%CARGO_PACKAGE_VERSION%%/${CARGO_PACKAGE_VERSION}/g" \
  conf-templates/Cargo.template.toml > src-tauri/Cargo.toml

echo "  Cargo.toml              -> patched [${CARGO_PACKAGE_NAME} ${CARGO_PACKAGE_VERSION}]"

# bridge.constants.json
jq --arg appUrl "$APP_URL_ORIGIN" '
  .appUrl = $appUrl
' conf-templates/bridge.constants.template.json > src-ts/bridge.constants.json

echo "  bridge.constants.json   -> patched"

# tauri.conf.json
cp "${TAURI_TEMPLATE}" src-tauri/tauri.conf.json

if [ "$APP_FRONTEND" = "companion" ]; then
  jq '.build.frontendDist = "../apps/companion/dist"' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
  echo "  frontend                -> apps/companion/dist"
elif [ "$APP_FRONTEND" = "bundled" ]; then
  # frontendDist is resolved from the directory holding tauri.conf.json.
  jq --arg dist "../$APP_FRONTEND_DIST" '.build.frontendDist = $dist' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
  echo "  frontend                -> $APP_FRONTEND_DIST"
fi

# -----------------------------
# Patch configuration values
# -----------------------------

echo "2. Checking if file exists and is valid JSON"
jq . src-tauri/tauri.conf.json >/dev/null

# Identifier and product name
echo "3. Patching identifier and product name"
jq \
  --arg ident "$APP_IDENTIFIER" \
  --arg prod "$MAIN_WINDOW_TITLE" \
  '
  .identifier = ($ident // .identifier) |
  .productName = ($prod // .productName)
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# Top-level version
echo "3.1. Patching top-level version"
jq \
  --arg ver "$APP_VERSION" \
  '.version = $ver' \
  src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# CSP allowlist
echo "5. Patching CSP allowlist"
if [ "$COMPANION_MODE" = "true" ]; then
  jq '
    .app = (.app // {}) |
    .app.security = (.app.security // {}) |
    .app.security.csp = (
      "default-src * data: blob: '\''unsafe-inline'\'' '\''unsafe-eval'\''; " +
      "img-src * data: blob:; " +
      "connect-src * ipc: http://ipc.localhost; " +
      "media-src *;"
    )
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
elif [ -n "$APP_URL_ORIGIN" ]; then
  jq --arg url "$APP_URL_ORIGIN" '
    .app = (.app // {}) |
    .app.security = (.app.security // {}) |
    .app.security.csp = (
      "default-src '\''self'\'' " + $url + "; " +
      "script-src '\''self'\'' " + $url + " '\''unsafe-inline'\''; " +
      "style-src '\''self'\'' " + $url + " '\''unsafe-inline'\''; " +
      "img-src * data: blob:; " +
      "connect-src *; " +
      "media-src *;"
    )
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
else
  jq '
    .app = (.app // {}) |
    .app.security = (.app.security // {}) |
    .app.security.csp = (
      "default-src '\''self'\''; " +
      "script-src '\''self'\'' '\''unsafe-inline'\''; " +
      "style-src '\''self'\'' '\''unsafe-inline'\''; " +
      "img-src '\''self'\'' data: blob:; " +
      "connect-src '\''self'\'' ipc: http://ipc.localhost; " +
      "media-src '\''self'\'' blob:;"
    )
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
fi

# Ensure remote capability is enabled
echo "5.1. Ensuring remote capability is enabled"
jq '
  .app = (.app // {}) |
  .app.security = (.app.security // {}) |
  .app.security.capabilities = (
    ((.app.security.capabilities // []) + ["remote"]) | unique
  )
' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# Updater config
if [ "$UPDATER_ENABLED" = "true" ]; then
  echo "6. Enabling developer-configured updater"
  jq --arg endpoint "$UPDATE_ENDPOINT" --arg pubkey "$ED25519_PUBKEY" '
    .bundle = (.bundle // {}) |
    .bundle.createUpdaterArtifacts = true |
    .plugins = (.plugins // {}) |
    .plugins.updater = {
      "endpoints": [$endpoint],
      "pubkey": $pubkey
    }
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp
  mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

  jq '
    .permissions = (((.permissions // []) + ["updater:default"]) | unique)
  ' src-tauri/capabilities/remote.json > src-tauri/capabilities/remote.json.tmp
  mv src-tauri/capabilities/remote.json.tmp src-tauri/capabilities/remote.json
  echo "  Build with Cargo feature: updater"
else
  echo "6. Updater disabled (default)"
  jq '
    .bundle = (.bundle // {}) |
    .bundle.createUpdaterArtifacts = false |
    .plugins = (.plugins // {}) |
    del(.plugins.updater)
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp
  mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

  jq '
    .permissions = ((.permissions // []) | map(select(. != "updater:default")))
  ' src-tauri/capabilities/remote.json > src-tauri/capabilities/remote.json.tmp
  mv src-tauri/capabilities/remote.json.tmp src-tauri/capabilities/remote.json
fi

# Deep link config
echo "7. Patching deep-link plugin configuration"
if [ -n "$DEEPLINK_SCHEME" ]; then
  jq --arg scheme "$DEEPLINK_SCHEME" '
    .plugins = (.plugins // {}) |
    .plugins["deep-link"] = (.plugins["deep-link"] // {}) |
    .plugins["deep-link"].desktop = (.plugins["deep-link"].desktop // {}) |
    .plugins["deep-link"].desktop.schemes = [ $scheme ]
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
fi

# -----------------------------
# Final checks
# -----------------------------

echo "8. Final checks"
echo "  tauri.conf.json -> patched"

echo "  enabled capabilities:"
jq '.app.security.capabilities' src-tauri/tauri.conf.json

echo "  final window URLs:"
jq '.app.windows[].url' src-tauri/tauri.conf.json

echo "  final remote.json:"
cat src-tauri/capabilities/remote.json
