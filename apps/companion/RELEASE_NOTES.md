Desktopr Companion is a desktop app built with Desktopr for trying the runtime
before you build your own app. Load any web app (for example
`localhost:3000`) with the whole Desktopr bridge available, or try the API in
the bridge playground.

## Download

| Platform | File |
| --- | --- |
| macOS | `.dmg` |
| Windows | [Microsoft Store](https://apps.microsoft.com/detail/9mtpbs9343t4) (recommended) or the `-setup.exe` installer |
| Linux | `.AppImage`, `.deb` or `.rpm` |

Each platform's `SHA256SUMS-<platform>.txt` lists the file checksums.

## First launch

### macOS

The app is signed and notarized. Open the `.dmg`, drag Desktopr Companion to
Applications and open it.

### Windows

The Microsoft Store version installs and updates without warnings.

The `-setup.exe` installer is not code signed, so Windows SmartScreen may show
"Windows protected your PC". Select **More info**, then **Run anyway**.

### Linux

- AppImage: make it executable (`chmod +x Desktopr*.AppImage`) and run it.
- Debian/Ubuntu: `sudo apt install ./<file>.deb`
- Fedora/RHEL: `sudo dnf install ./<file>.rpm`

## Using it

- **Load web app:** enter a URL such as `https://example.com` or
  `localhost:3000`. It opens in the main window with every Desktopr permission,
  so only load apps you trust. Restart the companion to return to the start
  screen; the last URL is remembered.
- **Bridge playground:** run snippets against the `Desktopr` API.
