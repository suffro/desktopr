# Window isolation and per-window capabilities

## Decision

The static `remote` capability (`capabilities/remote.json`) applies only to the
`main` window and webview. Windows created at runtime get their own capability
through Tauri's `Manager::add_capability` (`src-tauri/src/bridge/acl.rs`):

- `dtr_win_open` grants a copy of the `remote` capability (same remote origins
  and permissions) bound to the new window's label.
- `dtr_launch_companion` grants a reduced companion capability bound to the
  companion label: `desktopr-bridge` is replaced by
  `desktopr-bridge-companion`, and `autostart:*`, `updater:*` and
  `desktopr-bridge-debug` are dropped.

`desktopr-bridge-companion` allows the whole bridge except commands that open
windows or change app-wide state: `dtr_win_open`, `dtr_launch_companion`, WASM
module install/remove and plugin storage/job clearing, autostart changes,
diagnostics privacy/retention changes and diagnostics removal/clearing.
`scripts/check-bridge-permissions.mjs` keeps this list, the main list, the TS
bridge and `main.rs` consistent.

Filesystem scope is derived from the calling window that Tauri injects into
each command, never from frontend input. Companion windows (labels with the
reserved `dtr-cache-only-window-` prefix) are confined to their own scope and
cannot use plugin storage; every other window uses the main scope. A
`windowLabel` sent by the frontend must match. `dtr_win_open` refuses the
reserved prefix, and companion windows cannot open windows or companions.

`desktopr-bridge-debug` (the diagnostics test commands registered only in debug
builds) is granted only by the development configuration scripts.

## Rationale

- Tauri capabilities cannot exclude windows, so a `windows: ["*"]` capability
  would also cover companion windows and union with any reduced capability.
  Window labels are chosen by developers, so a label prefix for main windows
  would break the documented window API.
- Runtime capabilities keep the documented low-level `Desktopr.tauri` API
  working for trusted windows while limiting it for companion windows at the
  ACL level, not only in Rust command bodies.
- Before this change the frontend chose the filesystem scope, so any window
  could read or clear another window's data or the main scope.

## Verification

Unit tests cover scope derivation, companion permission filtering and the
generated capability. A local debug build ran a self-test page that opened a
child window and a companion and checked 24 allowed/denied calls (bridge and
raw plugin commands, label spoofing, plugin storage, debug commands, invalid
companion URL without leaked sandbox).
