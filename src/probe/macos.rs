use crate::model::{DefaultBrowser, Install, Report};
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn collect(_debug: bool) -> Result<Report> {
    todo!();
}

fn detect_default() -> Result<DefaultBrowser> {
    todo!();
}
