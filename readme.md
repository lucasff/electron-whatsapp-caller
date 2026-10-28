<p align="center">
  <img src="assets/logo.svg" width="128" height="128" alt="WhatsApp Caller logo">
</p>

<h1 align="center">WhatsApp Caller</h1>

<p align="center">
  Open any <code>tel:</code> or <code>callto:</code> link from any website in WhatsApp.
</p>

<p align="center">
  <a href="https://github.com/lucasff/electron-whatsapp-caller/actions/workflows/ci.yml"><img src="https://github.com/lucasff/electron-whatsapp-caller/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/lucasff/electron-whatsapp-caller/releases/latest"><img src="https://img.shields.io/github/v/release/lucasff/electron-whatsapp-caller" alt="Latest release"></a>
  <a href="license"><img src="https://img.shields.io/badge/license-MIT-green" alt="MIT license"></a>
</p>

Built with [Tauri](https://tauri.app) (Rust). The app is ~5 MB and has no window.
It opens only to start WhatsApp, then exits.

## Install

*macOS 10.15+, Linux, and Windows 10+ are supported.*

[**Download**](https://github.com/lucasff/electron-whatsapp-caller/releases/latest) the file for your system:

- **macOS**: `.dmg`
- **Linux**: `.AppImage` or `.deb` (the AppImage needs to be [made executable](http://discourse.appimage.org/t/how-to-make-an-appimage-executable/80))
- **Windows**: `.exe` installer

## Getting started

1. **Open the app once** to register the protocols. It opens the settings file and exits.
2. Set WhatsApp Caller as the default app for `tel:` / `callto:` links in your OS.
3. Optional: it also registers `whatsapp://send?phone=…` links. If you don't use the WhatsApp desktop app, make WhatsApp Caller the handler so these links open in your chosen `handler`.

The app doesn't stay running in the background. It opens only to start WhatsApp.

## Settings

Opening the app by hand opens `config.json`:

| OS | Location |
| --- | --- |
| macOS | `~/Library/Application Support/me.lucasfreitas.WhatsAppCaller/config.json` |
| Linux | `~/.config/me.lucasfreitas.WhatsAppCaller/config.json` |
| Windows | `%APPDATA%\me.lucasfreitas.WhatsAppCaller\config.json` |

```json
{
  "countryCodes": ["DE", "BR"],
  "handler": "auto",
  "ferdium": { "serviceId": "", "method": "deeplink", "cdpPort": 9222 }
}
```

`countryCodes` are tried in order for numbers without a `+` prefix.

`handler` chooses where links open:

| Handler | Opens |
| --- | --- |
| `auto` | WhatsApp desktop app, else the browser |
| `app` | `whatsapp://send?phone=…` |
| `web` | `https://api.whatsapp.com/send` |
| `ferdium` | WhatsApp service inside [Ferdium](https://github.com/ferdium/ferdium-app) |

If the chosen handler fails, the link opens in the browser.

For a `whatsapp:` link, `auto` and `app` open the browser, because `whatsapp://` would reach this app again. Use `web` or `ferdium` to pick the target.

`https://api.whatsapp.com/send?phone=…`, `https://web.whatsapp.com/send?phone=…` and `https://wa.me/…` links are parsed too, but the OS hands `https` links to the browser, not to this app. Route them here with a link router such as [Finicky](https://github.com/johnste/finicky), or pass them as an argument on Windows and Linux.

### Ferdium

Set `ferdium.serviceId` to the id of your WhatsApp service. List them with:

```sh
sqlite3 "~/Library/Application Support/Ferdium/server.sqlite" "select serviceId,name from services"
```

Then pick `ferdium.method`:

- `deeplink`: `ferdium://service/<id>?url=<encoded web.whatsapp.com url>`. Needs a Ferdium build with `?url=` support (not in a stock release yet).
- `cdp`: start Ferdium with `--remote-debugging-port=9222`. Works with stock Ferdium.
- `keystroke` (macOS): switches service and types the number into WhatsApp's "new chat". Needs Accessibility permission, experimental.

## Development

```sh
npm install
npm run dev     # run
npm test        # unit tests
npm run lint    # clippy + rustfmt
npm run build   # bundle for your OS
```

Requires [Rust](https://rustup.rs) and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).
Releases are built by GitHub Actions: push a `v*` tag to get a draft release with all installers.

---

## Credits

Built with [Tauri](https://tauri.app). The logo is in [`assets/logo.svg`](assets/logo.svg);
regenerate the app icons with `npx tauri icon <1024px png>`.
