# scripts/dev-conf.sh
set -euo pipefail

: "${APP_URL:=}"
: "${APP_VERSION:=0.2.1}"
: "${CARGO_PACKAGE_VERSION:=0.2.1}"
: "${CARGO_PACKAGE_NAME:=desktopr-wrapper}"

APP_URL_ORIGIN=""
cp conf-templates/remote.template.json src-tauri/capabilities/remote.json

if [ -n "$APP_URL" ]; then
  APP_URL_ORIGIN="$(printf '%s' "$APP_URL" | sed -E 's#^(https?://[^/]+).*$#\1#')"
  if ! printf '%s' "$APP_URL_ORIGIN" | grep -Eq '^https?://[^/]+$'; then
    echo "Invalid APP_URL: expected an absolute HTTP(S) URL"
    exit 1
  fi

  jq --arg url "${APP_URL_ORIGIN}/*" '
    .remote = { "urls": [$url] }
  ' src-tauri/capabilities/remote.json > src-tauri/capabilities/remote.json.tmp
  mv src-tauri/capabilities/remote.json.tmp src-tauri/capabilities/remote.json
fi


sed -e "s/%%CARGO_PACKAGE_NAME%%/${CARGO_PACKAGE_NAME}/g" \
    -e "s/%%CARGO_PACKAGE_VERSION%%/${CARGO_PACKAGE_VERSION}/g" \
  conf-templates/Cargo.template.toml > src-tauri/Cargo.toml


jq --arg appUrl "$APP_URL_ORIGIN" '
  .appUrl = $appUrl
' conf-templates/bridge.constants.template.json > src-ts/bridge.constants.json


cp conf-templates/tauri.conf.template.dev.json src-tauri/tauri.conf.json

cat > src-tauri/window.env << EOF
MAIN_WINDOW_URL=${APP_URL}
MAIN_WINDOW_TITLE=Desktopr Dev
MAIN_WINDOW_WIDTH=1200
MAIN_WINDOW_HEIGHT=800
MAIN_WINDOW_BG_COLOR=#171717
MAIN_WINDOW_RESIZABLE=true
MAIN_WINDOW_VISIBLE=false
MAIN_WINDOW_OPEN_FULLSCREEN=false
EOF

if [ -n "$APP_URL_ORIGIN" ]; then
  echo "Dev application origin -> ${APP_URL_ORIGIN}"
else
  echo "Dev application -> bundled standalone page"
fi
echo "Dev Cargo.toml package name -> ${CARGO_PACKAGE_NAME}"
echo "Dev Cargo.toml package version -> ${CARGO_PACKAGE_VERSION}"
echo "Dev window.env -> written"
