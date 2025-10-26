use serde::Serialize;

#[derive(Serialize)]
pub struct Report {
    pub platform: String,
    pub default_browser: DefaultBrowser,
    pub installs: Vec<Install>,
}

#[derive(Serialize, Default)]
pub struct DefaultBrowser {
    pub name: String,
    pub is_ff_default: bool,
    pub evidence: String,
}

#[derive(Serialize)]
pub struct Install {
    pub channel: String,   // release/beta/esr/nightly/unknown
    pub version: String,
    pub path: String,
    pub source: String,    // registry/plist/desktop/pkgmgr
}
