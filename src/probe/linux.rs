use anyhow::Result;
use std::{fs, path::Path};
use regex::Regex;
use crate::model::{Report, DefaultBrowser, Install};

/*
Installs: scan .desktop entries:

/usr/share/applications/firefox*.desktop, /var/lib/snapd/desktop/applications/, ~/.local/share/applications/

Pull Exec=, Name=, and try --version by running the binary if available (optional).

Classify channel from desktop file name (firefox-esr.desktop, firefox-beta.desktop, firefox-nightly.desktop) or path (/snap/firefox/…, /opt/firefox/).

Default browser: use xdg-settings get default-web-browser (fastest) and check if it contains firefox. If you want to avoid shelling out, parse the XDG mimeapps lists:

~/.config/mimeapps.list (or ~/.local/share/applications/mimeapps.list) → [Default Applications] entries for x-scheme-handler/http=https…
 */

pub fn collect(_debug: bool) -> Result<Report> {
    let platform = "linux".into();
    let installs = find_installs()?;
    let default_browser = detect_default()?;
    Ok(Report { platform, default_browser, installs })
}

fn find_installs() -> Result<Vec<Install>> {
    let mut out = Vec::new();
    let candidates = [
        "/usr/share/applications",
        "/var/lib/snapd/desktop/applications",
        "/usr/local/share/applications",
    ];
    let re = Regex::new(r"^firefox[^.]*\.desktop$")?;

    for dir in candidates {
        if Path::new(dir).is_dir() {
            for entry in fs::read_dir(dir)? {
                let p = entry?.path();
                if p.file_name().and_then(|s| s.to_str()).is_some_and(|s| re.is_match(s)) && 
                    let Some(inst) = parse_desktop(&p)
                {
                    out.push(inst);
                }
            }
        }
    }
    Ok(out)
}

fn parse_desktop(path: &Path) -> Option<Install> {
    let text = fs::read_to_string(path).ok()?;
    let exec = value_for_key(&text, "Exec")?;
    let _name = value_for_key(&text, "Name").unwrap_or("Firefox".into());
    let channel = if path.file_name()?.to_str()?.contains("esr") { "esr" }
                  else if path.file_name()?.to_str()?.contains("beta") { "beta" }
                  else if path.file_name()?.to_str()?.contains("nightly") { "nightly" }
                  else { "release" }.to_string();

    // Optional: try "--version"
    let version = get_version_from_exec(&exec).unwrap_or_else(|| "unknown".into());

    Some(Install {
        channel,
        version,
        path: exec,
        source: "desktop".into(),
    })
}

// Pulls the value for a given key from a multi-line key = value text block.
fn value_for_key(text: &str, key: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?m)^\s*{}\s*=\s*(.+)$", regex::escape(key))).ok()?;
    re.captures(text).and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string())
}

fn get_version_from_exec(exec_line: &str) -> Option<String> {
    // Exec like: "firefox %u" or "/snap/bin/firefox %u"
    let bin = exec_line.split_whitespace().next()?;
    let out = std::process::Command::new(bin).arg("--version").output().ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout);
    // "Mozilla Firefox 128.0.2"
    Some(s.split_whitespace().last()?.to_string())
}

fn detect_default() -> Result<DefaultBrowser> {
    // Try xdg-settings first
    if let Ok(out) = std::process::Command::new("xdg-settings")
        .args(["get", "default-web-browser"]).output() && out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("{s}");
            let is_ff = s.to_ascii_lowercase().contains("firefox");
            return Ok(DefaultBrowser {
                name: if is_ff { "Firefox".into() } else { s.strip_suffix(".desktop").unwrap_or("Other").to_string() },
                is_ff_default: is_ff,
                evidence: "xdg-settings".into(),
            });
    }

    // Fallback: parse mimeapps.list
    let paths = [
        dirs::config_dir().map(|p| p.join("mimeapps.list")),
        dirs::home_dir().map(|p| p.join(".local/share/applications/mimeapps.list")),
    ];
    for p in paths.into_iter().flatten() {
        if let Ok(text) = fs::read_to_string(p) && (text.contains("x-scheme-handler/http=firefox") || text.contains("=firefox.desktop")) {
            return Ok(DefaultBrowser { name: "Firefox".into(), is_ff_default: true, evidence: "mimeapps".into() });
        }
    }

    Ok(DefaultBrowser { name: "Other".into(), is_ff_default: false, evidence: "none".into() })
}
