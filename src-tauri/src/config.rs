use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Handler {
    /// WhatsApp desktop app, else the browser.
    #[default]
    Auto,
    App,
    Web,
    Ferdium,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum FerdiumMethod {
    /// `ferdium://service/<id>?url=...` (needs a Ferdium build with `?url=` support).
    #[default]
    Deeplink,
    /// Chrome DevTools Protocol (Ferdium started with `--remote-debugging-port`).
    Cdp,
    /// macOS only: types the number into WhatsApp's "new chat".
    Keystroke,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct FerdiumConfig {
    /// Ferdium service id of the WhatsApp service.
    pub service_id: String,
    pub method: FerdiumMethod,
    pub cdp_port: u16,
}

impl Default for FerdiumConfig {
    fn default() -> Self {
        Self {
            service_id: String::new(),
            method: FerdiumMethod::default(),
            cdp_port: 9222,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    /// Country codes tried, in order, for numbers without an international prefix.
    pub country_codes: Vec<String>,
    pub handler: Handler,
    pub ferdium: FerdiumConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            country_codes: vec!["DE".into(), "BR".into()],
            handler: Handler::default(),
            ferdium: FerdiumConfig::default(),
        }
    }
}

impl Config {
    /// Loads the config file, writing the defaults first when it does not exist.
    /// An unreadable file falls back to the defaults and is left untouched.
    pub fn load(path: &PathBuf) -> Self {
        match fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|error| {
                eprintln!("Invalid config {}: {error}", path.display());
                Self::default()
            }),
            Err(_) => {
                let config = Self::default();
                if let Some(dir) = path.parent() {
                    let _ = fs::create_dir_all(dir);
                }
                if let Ok(json) = serde_json::to_string_pretty(&config) {
                    let _ = fs::write(path, json + "\n");
                }
                config
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_config_uses_defaults() {
        let config: Config =
            serde_json::from_str(r#"{"handler":"ferdium","ferdium":{"method":"cdp"}}"#).unwrap();
        assert_eq!(config.handler, Handler::Ferdium);
        assert_eq!(config.ferdium.method, FerdiumMethod::Cdp);
        assert_eq!(config.ferdium.cdp_port, 9222);
        assert_eq!(config.country_codes, vec!["DE", "BR"]);
    }
}
