# scripts/prod-conf.sh
set -euo pipefail

# -----------------------------
# Read inputs (with safe defaults)
# -----------------------------
: "${APP_URL:=http://blank.html}"
: "${APP_VERSION:=0.1.0}"
: "${CARGO_PACKAGE_NAME:=bubbledesk-wrapper}"
: "${CARGO_PACKAGE_VERSION:=$APP_VERSION}"

cp conf-templates/tauri.conf.template.sidecarbin.json src-tauri/tauri.conf.json
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
