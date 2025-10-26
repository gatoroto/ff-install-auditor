mod model;
mod probe;

use anyhow::Result;
use clap::Parser;
use model::Report;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    pretty: bool,
    #[arg(long)]
    machine: bool,
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let report: Report = probe::collect(args.debug)?;

    let json = if args.pretty {
        serde_json::to_string_pretty(&report)?
    } else {
        serde_json::to_string(&report)?
    };
    println!("{}", json);

    if args.machine {
        // 0 ok, 1 none found, 2 not default, 3 error
        if report.installs.is_empty() {
            std::process::exit(1);
        }
        if !report.default_browser.is_ff_default {
            std::process::exit(2);
        }
    }
    Ok(())
}
