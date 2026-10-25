use std::io;

use winreg::{enums::HKEY_CURRENT_USER, RegKey};

/// Registers the app in Windows "Default apps" for tel: and callto: links.
pub fn register(exe: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let (capabilities, _) = hkcu.create_subkey("Software\\WhatsApp Caller\\Capabilities")?;
    capabilities.set_value("ApplicationName", &"WhatsApp Caller")?;
    capabilities.set_value(
        "ApplicationDescription",
        &"Open tel: and callto: links in WhatsApp",
    )?;

    let (associations, _) =
        hkcu.create_subkey("Software\\WhatsApp Caller\\Capabilities\\URLAssociations")?;
    for scheme in ["tel", "callto"] {
        let prog_id = format!("WhatsApp Caller.{scheme}");
        associations.set_value(scheme, &prog_id)?;

        let class = format!("Software\\Classes\\{prog_id}");
        let (icon, _) = hkcu.create_subkey(format!("{class}\\DefaultIcon"))?;
        icon.set_value("", &exe)?;
        let (command, _) = hkcu.create_subkey(format!("{class}\\shell\\open\\command"))?;
        command.set_value("", &format!("\"{exe}\" \"%1\""))?;
    }

    let (registered, _) = hkcu.create_subkey("Software\\RegisteredApplications")?;
    registered.set_value(
        "WhatsApp Caller",
        &"Software\\WhatsApp Caller\\Capabilities",
    )?;
    Ok(())
}
