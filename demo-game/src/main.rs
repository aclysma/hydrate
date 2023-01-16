use std::path::PathBuf;

pub fn build_data_source_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../demo-editor/data/build_data"))
}

fn main() {
    // Setup logging
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    hydrate::loader::Loader::new(build_data_source_path());
}
