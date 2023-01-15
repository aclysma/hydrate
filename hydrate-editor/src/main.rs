fn main() {
    // Setup logging
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    hydrate_editor::run();
}

// fn main() {
//     let walker = globwalk::GlobWalkerBuilder::from_patterns("/Users/pmd/dev/rust/m3/m3/data/test_project", &["**.ta"]).build().unwrap();
//
//     for i in walker {
//         println!("{:?}", i.unwrap());
//     }
// }
