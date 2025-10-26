use anyhow::Result;
use crate::model::Report;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

pub fn collect(debug: bool) -> Result<Report> {
    #[cfg(target_os = "windows")]
    { windows::collect(debug) }
    #[cfg(target_os = "macos")]
    { macos::collect(debug) }
    #[cfg(target_os = "linux")]
    { linux::collect(debug) }
}
