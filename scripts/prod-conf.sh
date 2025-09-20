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
: "${UPDATE_ENDPOINT:?Missing UPDATE_ENDPOINT (set by CI)}}"
: "${ED25519_PUBKEY:?Missing ED25519_PUBKEY (GitHub secret)}}"
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
echo "Generate files from templates"

# 1) remote.json (capabilities)
sed -e "s|%%APP_URL%%|${APP_URL}|g" \
    -e "s|%%ASSETS_CDN_URL%%|${APP_URL}|g" \
  conf-templates/remote.template.json > src-tauri/capabilities/remote.json

# 2) Cargo.toml
sed -e "s/%%CARGO_PACKAGE_NAME%%/${CARGO_PACKAGE_NAME}/g" \
    -e "s/%%CARGO_PACKAGE_VERSION%%/${CARGO_PACKAGE_VERSION}/g" \
  conf-templates/Cargo.template.toml > src-tauri/Cargo.toml

# 3) bridge.constants.json
sed -e "s|%%APP_URL%%|${APP_URL}|g" \
    -e "s|%%APP_VERSION%%|${APP_VERSION}|g" \
  conf-templates/bridge.constants.template.json > src-ts/bridge.constants.json

# 4) tauri.conf.json (start from PROD template, then patch via jq for dynamic fields)
cp "${TAURI_TEMPLATE}" src-tauri/tauri.conf.json

# Ensure file exists and is valid JSON
echo "Ensure file exists and is valid JSON"
jq . src-tauri/tauri.conf.json >/dev/null

# Identifier & product name
echo "Identifier & product name"
jq \
  --arg ident "$APP_IDENTIFIER" \
  --arg prod "$MAIN_WINDOW_TITLE" \
  '
  .identifier = ($ident // .identifier) |
  .productName = ($prod // .productName)
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# Windows config
echo "Windows config"
jq \
  --arg title "$MAIN_WINDOW_TITLE" \
  --argjson width "$MAIN_WINDOW_WIDTH" \
  --argjson height "$MAIN_WINDOW_HEIGHT" \
  --arg bg "$MAIN_WINDOW_BG_COLOR" \
  --arg url "$MAIN_WINDOW_URL" \
  --argjson resizable "$( [ "$MAIN_WINDOW_RESIZABLE" = "true" ] && echo true || echo false )" \
  --argjson fullscreen "$( [ "$MAIN_WINDOW_OPEN_FULLSCREEN" = "true" ] && echo true || echo false )" \
  '
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
echo "CSP allowlist for APP_URL"
jq --arg url "$APP_URL" '
  .app.security.csp = ("default-src '\''self'\'' " + $url + "; script-src '\''self'\'' " + $url + " '\''unsafe-inline'\''; style-src '\''self'\'' " + $url + " '\''unsafe-inline'\''; img-src * data: blob:; connect-src *; media-src *;")
' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# Updater (required in CI)
echo "Updater (required in CI)"
jq --arg endpoint "$UPDATE_ENDPOINT" --arg pubkey "$ED25519_PUBKEY" '
  .updater = (.updater // {}) |
  .updater.active = true |
  .updater.endpoints = [ $endpoint ] |
  .updater.pubkey = $pubkey
' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# Deeplink (optional)
echo "Deeplink (optional)"
if [ -n "$DEEPLINK_SCHEME" ]; then
  jq --arg scheme "$DEEPLINK_SCHEME" '
    .app.protocols = (.app.protocols // {}) |
    .app.protocols.custom = [ { "name": $scheme, "scheme": $scheme } ]
  ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
fi

echo "PROD remote.json  -> ${APP_URL}"
echo "PROD Cargo.toml   -> ${CARGO_PACKAGE_NAME} ${CARGO_PACKAGE_VERSION}"
echo "PROD tauri.conf   -> patched"