use crate::model::Report;
use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn collect(debug: bool) -> Result<Report> {
    #[cfg(target_os = "windows")]
    {
        windows::collect(debug)
    }
    #[cfg(target_os = "macos")]
    {
        macos::collect(debug)
    }
    #[cfg(target_os = "linux")]
    {
        linux::collect(debug)
    }
}
