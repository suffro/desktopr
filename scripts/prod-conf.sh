# scripts/prod-conf.sh
set -euo pipefail

# -----------------------------
# Read inputs (with safe defaults)
# -----------------------------
: "${APP_URL:=http://blank.html}"
: "${APP_VERSION:=0.1.0}"
: "${CARGO_PACKAGE_NAME:=bubbledesk-wrapper}"
: "${CARGO_PACKAGE_VERSION:=$APP_VERSION}"
: "${APP_IDENTIFIER:=app.bubbledesk.app}"
: "${UPDATE_ENDPOINT:?Missing UPDATE_ENDPOINT (set by CI)}"
: "${TAURI_SIGNING_PUBLIC_KEY:=}"
if [ -z "${ED25519_PUBKEY:-}" ] && [ -n "${TAURI_SIGNING_PUBLIC_KEY:-}" ]; then
  ED25519_PUBKEY="$TAURI_SIGNING_PUBLIC_KEY"
fi
: "${ED25519_PUBKEY:?Missing ED25519_PUBKEY (CI var/secret)}"
: "${DEEPLINK_SCHEME:=}"
: "${MAIN_WINDOW_TITLE:=Bubbledesk}"
: "${MAIN_WINDOW_WIDTH:=1200}"
: "${MAIN_WINDOW_HEIGHT:=800}"
: "${MAIN_WINDOW_BG_COLOR:=#ffffff}"
: "${MAIN_WINDOW_URL:=$APP_URL}"
: "${MAIN_WINDOW_RESIZABLE:=true}"
: "${MAIN_WINDOW_OPEN_FULLSCREEN:=false}"

# -----------------------------
# Always use PROD templates
# -----------------------------

TAURI_TEMPLATE="conf-templates/tauri.conf.template.prod.json"

# -----------------------------
# Generate files from templates
# -----------------------------
echo "1. Generating files from templates"
# remote.json (capabilities)
sed -e "s|%%APP_URL%%|${APP_URL}|g" \
    -e "s|%%ASSETS_CDN_URL%%|${APP_URL}|g" \
  conf-templates/remote.template.json > src-tauri/capabilities/remote.json
echo "  remote.json             -> patched [${APP_URL}]"

# Cargo.toml
sed -e "s/%%CARGO_PACKAGE_NAME%%/${CARGO_PACKAGE_NAME}/g" \
    -e "s/%%CARGO_PACKAGE_VERSION%%/${CARGO_PACKAGE_VERSION}/g" \
  conf-templates/Cargo.template.toml > src-tauri/Cargo.toml
echo "  Cargo.toml              -> patched [${CARGO_PACKAGE_NAME} ${CARGO_PACKAGE_VERSION}]"

# bridge.constants.json
sed -e "s|%%APP_URL%%|${APP_URL}|g" \
    -e "s|%%APP_VERSION%%|${APP_VERSION}|g" \
  conf-templates/bridge.constants.template.json > src-ts/bridge.constants.json
echo "  bridge.constants.json   -> patched"

# tauri.conf.json (start from PROD template, then patch via jq for dynamic fields)
cp "${TAURI_TEMPLATE}" src-tauri/tauri.conf.json

# -----------------------------
# Patch configuration values
# -----------------------------

# Ensure file exists and is valid JSON
echo "2. Checking if file exists and is valid JSON"
jq . src-tauri/tauri.conf.json >/dev/null

# Identifier & product name
echo "3. Patching identifier and product name"
jq \
  --arg ident "$APP_IDENTIFIER" \
  --arg prod "$MAIN_WINDOW_TITLE" \
  '
  .identifier = ($ident // .identifier) |
  .productName = ($prod // .productName)
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# Ensure top-level app version is set from APP_VERSION
echo "3.1. Patching top-level version"
jq \
  --arg ver "$APP_VERSION" \
  '.version = $ver' \
  src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# Windows config
echo "4. Patching app windows configuration"
jq \
  --arg title "$MAIN_WINDOW_TITLE" \
  --argjson width "$MAIN_WINDOW_WIDTH" \
  --argjson height "$MAIN_WINDOW_HEIGHT" \
  --arg bg "$MAIN_WINDOW_BG_COLOR" \
  --arg url "$MAIN_WINDOW_URL" \
  --argjson resizable "$( [ "$MAIN_WINDOW_RESIZABLE" = "true" ] && echo true || echo false )" \
  --argjson fullscreen "$( [ "$MAIN_WINDOW_OPEN_FULLSCREEN" = "true" ] && echo true || echo false )" \
  '
  .app = (.app // {}) |
  .app.windows = (
    if (.app.windows | type) == "array" and (.app.windows | length) > 0 then
      (.app.windows | map(
        if .label == "main" or .label == "main-window" then
          .title= $title | .width=$width | .height=$height | .resizable=$resizable | .fullscreen=$fullscreen | .backgroundColor=$bg | .url=$url
        else
          .
        end
      ))
    else
      [
        {"label":"main","title":$title,"width":$width,"height":$height,"resizable":$resizable,"fullscreen":$fullscreen,"backgroundColor":$bg,"url":$url}
      ]
    end
  )
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# CSP allowlist for APP_URL
echo "5. Patching CSP allowlist for APP_URL"
jq --arg url "$APP_URL" '
  .app.security.csp = ("default-src '\''self'\'' " + $url + "; script-src '\''self'\'' " + $url + " '\''unsafe-inline'\''; style-src '\''self'\'' " + $url + " '\''unsafe-inline'\''; img-src * data: blob:; connect-src *; media-src *;")
' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

 # Updater (Tauri v2 plugin)
 echo "6. Patching updater settings (Tauri v2 plugin)"
 jq --arg endpoint "$UPDATE_ENDPOINT" --arg pubkey "$ED25519_PUBKEY" '
   .bundle = (.bundle // {}) |
   .bundle.createUpdaterArtifacts = true |
   .plugins = (.plugins // {}) |
 	.plugins.updater = (.plugins.updater // {}) |
   .plugins.updater.endpoints = [ $endpoint ] |
   .plugins.updater.pubkey = $pubkey
 ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json


# Deeplink (optional) — Tauri v2 deep-link plugin
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
# Generate Companion configuration clone (generic)
# -----------------------------
echo "8. Generating tauri.conf.companion.json"

# Derive dynamic values
COMPANION_IDENTIFIER="${APP_IDENTIFIER}.companion"
COMPANION_PRODUCT_NAME="${MAIN_WINDOW_TITLE} Companion"
COMPANION_DEEPLINK_SCHEME="${DEEPLINK_SCHEME:-${APP_NAME:-app}}-companion"

jq \
  --arg ident "$COMPANION_IDENTIFIER" \
  --arg prod "$COMPANION_PRODUCT_NAME" \
  --arg scheme "$COMPANION_DEEPLINK_SCHEME" \
  '
  .productName = $prod |
  .identifier = $ident |
  .app.windows = [
    {
      "label": "main",
      "title": $prod,
      "url": "index.html",
      "fullscreen": false,
      "resizable": true
    }
  ] |
  .plugins["deep-link"] = {
    "desktop": { "schemes": [ $scheme ] }
  } |
  # Disable updater for companion
  del(.plugins.updater)
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.companion.json

# Validate JSON
jq . src-tauri/tauri.conf.companion.json >/dev/null

echo "  tauri.conf.companion.json -> created for [$COMPANION_PRODUCT_NAME / $COMPANION_IDENTIFIER]"
echo "  tauri.conf.json -> patched"
