use log::LevelFilter;
use std::path::PathBuf;
use structopt::StructOpt;

use hydrate_codegen::*;

fn main() -> Result<(), String> {
    let args = HydrateCodegenArgs::from_args();

    // Setup logging
    let level = if args.trace {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .default_format_timestamp_nanos(true)
        .filter_level(level)
        .init();

    if let Err(e) = run(&PathBuf::from(env!("CARGO_MANIFEST_DIR")), &args) {
        eprintln!("{}", e.to_string());
        Err("Hydrate codegen failed".to_string())
    } else {
        Ok(())
    }
}
