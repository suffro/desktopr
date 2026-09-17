# scripts/local-dev-conf.sh
set -euo pipefail

APP_VERSION="0.2.1"
CARGO_PACKAGE_VERSION="0.2.1"
CARGO_PACKAGE_NAME="desktopr-wrapper"

cp conf-templates/remote.template.json src-tauri/capabilities/remote.json
# Development builds also grant the debug-only diagnostics test commands.
jq '.permissions += ["desktopr-bridge-debug"]' src-tauri/capabilities/remote.json > src-tauri/capabilities/remote.json.tmp
mv src-tauri/capabilities/remote.json.tmp src-tauri/capabilities/remote.json


sed -e "s/%%CARGO_PACKAGE_NAME%%/${CARGO_PACKAGE_NAME}/g" \
    -e "s/%%CARGO_PACKAGE_VERSION%%/${CARGO_PACKAGE_VERSION}/g" \
  conf-templates/Cargo.template.toml > src-tauri/Cargo.toml


jq --arg appUrl "" '
  .appUrl = $appUrl
' conf-templates/bridge.constants.template.json > src-ts/bridge.constants.json


cp conf-templates/tauri.conf.template.local.dev.json src-tauri/tauri.conf.json

cat > src-tauri/window.env << 'EOF'
MAIN_WINDOW_URL=
MAIN_WINDOW_TITLE=Desktopr Companion
MAIN_WINDOW_WIDTH=1200
MAIN_WINDOW_HEIGHT=800
MAIN_WINDOW_BG_COLOR=#171717
MAIN_WINDOW_RESIZABLE=true
MAIN_WINDOW_VISIBLE=true
MAIN_WINDOW_OPEN_FULLSCREEN=false
EOF

echo "Local dev capability -> local frontend only"
echo "Dev Cargo.toml package name -> ${CARGO_PACKAGE_NAME}"
echo "Dev Cargo.toml package version -> ${CARGO_PACKAGE_VERSION}"
echo "Dev window.env -> written"
