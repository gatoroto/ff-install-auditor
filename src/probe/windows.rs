use crate::model::{DefaultBrowser, Install, Report};
use anyhow::{Context, Result};
use winreg::RegKey;
use winreg::enums::*;

pub fn collect(debug: bool) -> Result<Report> {
    let platform = "windows".to_string();
    let installs = find_installs(debug)?;
    let default_browser = detect_default(debug)?;

    Ok(Report {
        platform,
        default_browser,
        installs,
    })
}

fn find_installs(_debug: bool) -> Result<Vec<Install>> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut out = Vec::new();

    for base in &[
        r"SOFTWARE\Mozilla\Mozilla Firefox",
        r"SOFTWARE\WOW6432Node\Mozilla\Mozilla Firefox",
    ] {
        if let Ok(key) = hklm.open_subkey(base) {
            // CurrentVersion
            if let Ok(cur) = key.get_value("CurrentVersion") {
                let main = key.open_subkey(format!(r"{}\Main", cur))?;
                let path: String = main.get_value("Install Directory")?;
                // DisplayVersion may be in Uninstall key; try Main fallback
                let ver: String = main.get_value("Version").unwrap_or_else(|_| cur.clone());
                out.push(Install {
                    channel: classify_channel(&path, &ver),
                    version: ver,
                    path,
                    source: "registry".into(),
                });
            }
        }
    }
    Ok(out)
}

fn detect_default(_debug: bool) -> Result<DefaultBrowser> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let prog = |assoc: &str| -> Option<String> {
        hkcu.open_subkey(format!(
            r"SOFTWARE\Microsoft\Windows\Shell\Associations\UrlAssociations\{}\UserChoice",
            assoc
        ))
        .ok()
        .and_then(|k| k.get_value::<String, _>("ProgId").ok())
    };
    let http = prog("http");
    let https = prog("https");

    let is_ff = |p: &Option<String>| p.as_ref().map_or(false, |s| s.starts_with("FirefoxURL"));

    Ok(DefaultBrowser {
        name: if is_ff(&http) && is_ff(&https) {
            "Firefox".into()
        } else {
            "Other".into()
        },
        is_default: is_ff(&http) && is_ff(&https),
        evidence: "registry".into(),
    })
}

fn classify_channel(path: &str, ver: &str) -> String {
    let p = path.to_ascii_lowercase();
    if p.contains("nightly") {
        "nightly".into()
    } else if p.contains("esr") {
        "esr".into()
    } else if p.contains("beta") {
        "beta".into()
    } else if ver.contains("esr") {
        "esr".into()
    } else {
        "release".into()
    }
}
