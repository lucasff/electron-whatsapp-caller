use std::{thread, time::Duration};

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tungstenite::{connect, stream::MaybeTlsStream, Message};

use crate::config::{Config, FerdiumConfig, FerdiumMethod, Handler};

type Result<T> = std::result::Result<T, String>;

const WEB_ORIGIN: &str = "https://web.whatsapp.com";

fn web_url(number: &str) -> String {
    format!("{WEB_ORIGIN}/send?phone={number}")
}

fn open(app: &AppHandle, url: &str) -> Result<()> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

fn open_in_browser(app: &AppHandle, number: &str) -> Result<()> {
    open(
        app,
        &format!("https://api.whatsapp.com/send/?phone={number}"),
    )
}

// Opening an unregistered scheme fails on macOS and Linux. Windows shows a
// "get an app" dialog instead, so check the registry first.
#[cfg(windows)]
fn scheme_registered(scheme: &str) -> bool {
    use winreg::{enums::HKEY_CLASSES_ROOT, RegKey};
    RegKey::predef(HKEY_CLASSES_ROOT)
        .open_subkey(scheme)
        .is_ok()
}

#[cfg(not(windows))]
fn scheme_registered(_scheme: &str) -> bool {
    true
}

fn open_in_desktop_app(app: &AppHandle, number: &str) -> Result<()> {
    if !scheme_registered("whatsapp") {
        return Err("WhatsApp desktop app is not installed".into());
    }
    open(app, &format!("whatsapp://send?phone={number}"))
}

fn encode(value: &str) -> String {
    percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC).to_string()
}

fn ferdium_deeplink(app: &AppHandle, ferdium: &FerdiumConfig, number: &str) -> Result<()> {
    open(
        app,
        &format!(
            "ferdium://service/{}?url={}",
            ferdium.service_id,
            encode(&web_url(number))
        ),
    )
}

/// Sends one CDP command and returns its `result`.
fn cdp_send(ws_url: &str, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    let (mut socket, _) = connect(ws_url).map_err(|e| e.to_string())?;
    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    }
    let request = serde_json::json!({ "id": 1, "method": method, "params": params });
    socket
        .send(Message::text(request.to_string()))
        .map_err(|e| e.to_string())?;

    loop {
        let message = socket.read().map_err(|e| e.to_string())?;
        let Message::Text(text) = message else {
            continue;
        };
        let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if value["id"] != 1 {
            continue;
        }
        let _ = socket.close(None);
        return match value.get("error") {
            Some(error) => Err(error["message"].as_str().unwrap_or("CDP error").to_string()),
            None => Ok(value["result"].clone()),
        };
    }
}

/// Ferdium must run with `--remote-debugging-port=<cdpPort>`. Several WhatsApp
/// services can exist, so first switch to the configured one, then pick the
/// WhatsApp webview that is visible and focused.
fn ferdium_cdp(app: &AppHandle, ferdium: &FerdiumConfig, number: &str) -> Result<()> {
    if !ferdium.service_id.is_empty() {
        open(app, &format!("ferdium://service/{}", ferdium.service_id))?;
        thread::sleep(Duration::from_millis(500));
    }

    let list_url = format!("http://127.0.0.1:{}/json", ferdium.cdp_port);
    let targets: serde_json::Value = ureq::get(&list_url)
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;

    let ws_urls: Vec<&str> = targets
        .as_array()
        .into_iter()
        .flatten()
        .filter(|t| t["url"].as_str().is_some_and(|u| u.starts_with(WEB_ORIGIN)))
        .filter_map(|t| t["webSocketDebuggerUrl"].as_str())
        .collect();
    let Some(&first) = ws_urls.first() else {
        return Err("No WhatsApp Web webview found in Ferdium".into());
    };

    let mut chosen = first;
    if ws_urls.len() > 1 {
        for ws_url in &ws_urls {
            let result = cdp_send(
                ws_url,
                "Runtime.evaluate",
                serde_json::json!({
                    "expression": "document.visibilityState === 'visible' && document.hasFocus()",
                    "returnByValue": true
                }),
            )?;
            if result["result"]["value"] == true {
                chosen = ws_url;
                break;
            }
        }
    }

    cdp_send(
        chosen,
        "Page.navigate",
        serde_json::json!({ "url": web_url(number) }),
    )
    .map(|_| ())
}

/// macOS only: switch to the service, open "new chat" (Cmd+Ctrl+N) and type the
/// number. Needs the Accessibility permission. Press Return to pick the
/// "Message +number" result.
#[cfg(target_os = "macos")]
fn ferdium_keystroke(app: &AppHandle, ferdium: &FerdiumConfig, number: &str) -> Result<()> {
    open(app, &format!("ferdium://service/{}", ferdium.service_id))?;
    let script = format!(
        r#"delay 0.6
tell application "Ferdium" to activate
delay 0.4
tell application "System Events"
  keystroke "n" using {{command down, control down}}
  delay 0.6
  keystroke "+{number}"
end tell"#
    );
    let status = std::process::Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map_err(|e| e.to_string())?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "osascript failed".into())
}

#[cfg(not(target_os = "macos"))]
fn ferdium_keystroke(_app: &AppHandle, _ferdium: &FerdiumConfig, _number: &str) -> Result<()> {
    Err("keystroke method is macOS only".into())
}

fn open_in_ferdium(app: &AppHandle, ferdium: &FerdiumConfig, number: &str) -> Result<()> {
    match ferdium.method {
        FerdiumMethod::Deeplink => ferdium_deeplink(app, ferdium, number),
        FerdiumMethod::Cdp => ferdium_cdp(app, ferdium, number),
        FerdiumMethod::Keystroke => ferdium_keystroke(app, ferdium, number),
    }
}

/// Opens a WhatsApp chat using the configured handler. Falls back to the
/// browser when the chosen handler fails. `from_whatsapp_scheme` is set when
/// this app received a `whatsapp:` link: opening `whatsapp://` again would
/// reach this app, so the desktop app handlers use the browser instead.
pub fn open_chat(app: &AppHandle, config: &Config, number: &str, from_whatsapp_scheme: bool) {
    let result = match config.handler {
        Handler::Ferdium => open_in_ferdium(app, &config.ferdium, number),
        Handler::Web => open_in_browser(app, number),
        Handler::App | Handler::Auto if from_whatsapp_scheme => open_in_browser(app, number),
        Handler::App => open_in_desktop_app(app, number),
        Handler::Auto => open_in_desktop_app(app, number).or_else(|_| open_in_browser(app, number)),
    };

    if let Err(error) = result {
        eprintln!("Handler {:?} failed: {error}", config.handler);
        if let Err(error) = open_in_browser(app, number) {
            eprintln!("Browser fallback failed: {error}");
        }
    }
}
