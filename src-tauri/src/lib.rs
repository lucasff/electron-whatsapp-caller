mod config;
mod launcher;
mod phone;
#[cfg(windows)]
mod windows;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use config::Config;
use tauri::{AppHandle, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;

// The app only lives long enough to hand a link over. Launched without a
// link it opens the config file instead.
const LINK_GRACE_PERIOD: Duration = Duration::from_secs(2);

fn handle_link(app: &AppHandle, config_path: &PathBuf, link: &str) {
    let config = Config::load(config_path);
    match phone::parse_number(link, &config.country_codes) {
        Some(number) => launcher::open_chat(app, &config, &number, phone::is_whatsapp_scheme(link)),
        None => eprintln!("No valid phone number in link: {link}"),
    }
}

fn handle_links(
    app: &AppHandle,
    config_path: PathBuf,
    handled: &Arc<AtomicBool>,
    links: Vec<String>,
) {
    let Some(link) = links.into_iter().next() else {
        return;
    };
    handled.store(true, Ordering::SeqCst);

    let app = app.clone();
    thread::spawn(move || {
        handle_link(&app, &config_path, &link);
        app.exit(0);
    });
}

pub fn run() {
    tauri::Builder::default()
        // With the `deep-link` feature, links of a second launch reach `on_open_url`.
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let config_path = app.path().app_config_dir()?.join("config.json");
            let handled = Arc::new(AtomicBool::new(false));

            #[cfg(windows)]
            if let Err(error) = windows::register(&std::env::current_exe()?.to_string_lossy()) {
                eprintln!("Registering default app entries failed: {error}");
            }
            #[cfg(any(windows, target_os = "linux"))]
            app.deep_link().register_all()?;

            let to_strings =
                |urls: Vec<tauri::Url>| urls.into_iter().map(String::from).collect::<Vec<_>>();

            if let Some(urls) = app.deep_link().get_current()? {
                handle_links(
                    app.handle(),
                    config_path.clone(),
                    &handled,
                    to_strings(urls),
                );
            }

            let handle = app.handle().clone();
            let path = config_path.clone();
            let seen = handled.clone();
            app.deep_link().on_open_url(move |event| {
                handle_links(&handle, path.clone(), &seen, to_strings(event.urls()));
            });

            let handle = app.handle().clone();
            thread::spawn(move || {
                thread::sleep(LINK_GRACE_PERIOD);
                if !handled.load(Ordering::SeqCst) {
                    let _ = Config::load(&config_path);
                    let _ = handle
                        .opener()
                        .open_path(config_path.to_string_lossy(), None::<&str>);
                    // Give the editor a moment to receive the file before exiting.
                    thread::sleep(Duration::from_millis(500));
                    handle.exit(0);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running WhatsApp Caller");
}
