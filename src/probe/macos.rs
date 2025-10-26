use anyhow::{Context, Result};
use plist::Value;
use std::{fs, path::Path};
use crate::model::{Report, DefaultBrowser, Install};

pub fn collect(_debug: bool) -> Result<Report> {
    let platform = "macos".to_string();
    let mut installs = Vec::new();

    for app in &[
        "/Applications/Firefox.app",
        "/Applications/Firefox Nightly.app",
        "/Applications/Firefox Developer Edition.app",
    ] {
        if Path::new(app).exists() {
            if let Some((version, channel)) = read_app(app) {
                installs.push(Install {
                    channel,
                    version,
                    path: app.to_string(),
                    source: "plist".into(),
                });
            }
        }
    }

    let default_browser = detect_default()?;
    Ok(Report { platform, default_browser, installs })
}

fn read_app(app_path: &str) -> Option<(String, String)> {
    let info = format!("{}/Contents/Info.plist", app_path);
    let bytes = fs::read(info).ok()?;
    let plist = Value::from_reader_xml(bytes.as_slice()).ok()?;
    let ver = plist.as_dictionary()?
        .get("CFBundleShortVersionString")?
        .as_string()?.to_string();

    let bundle = plist.as_dictionary()?.get("CFBundleIdentifier")?.as_string()?.to_string();
    let channel = if bundle.contains("nightly") { "nightly" }
                  else if bundle.contains("dev") { "beta" } // Dev Edition ≈ beta channel
                  else { "release" }.to_string();

    Some((ver, channel))
}

fn detect_default() -> Result<DefaultBrowser> {
    // Parse LSHandlers plist (secure domain)
    let path = dirs::home_dir()
        .map(|p| p.join("Library/Preferences/com.apple.LaunchServices/com.apple.launchservices.secure.plist"))
        .ok_or_else(|| anyhow::anyhow!("no HOME"))?;

    let bytes = std::fs::read(&path).context("read launchservices plist")?;
    let plist = Value::from_reader_xml(bytes.as_slice()).context("parse plist")?;
    let handlers = plist.as_dictionary()
        .and_then(|d| d.get("LSHandlers"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("no LSHandlers"))?;

    let mut bundle = String::new();
    for h in handlers {
        if let Some(dict) = h.as_dictionary() {
            let scheme = dict.get("LSHandlerURLScheme").and_then(|v| v.as_string()).unwrap_or("");
            if scheme == "http" {
                if let Some(id) = dict.get("LSHandlerRoleAll").and_then(|v| v.as_string()) {
                    bundle = id.to_string();
                    break;
                }
            }
        }
    }

    let is_ff = bundle.starts_with("org.mozilla.firefox");
    Ok(DefaultBrowser {
        name: if is_ff { "Firefox".into() } else { "Other".into() },
        is_default: is_ff,
        evidence: "launchservices".into(),
    })
}
