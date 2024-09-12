fn main() {
    // Emit logging to either OSLog or stderr, depending on if using Mac
    // Catalyst or native.
    // TODO: If running Mac Catalyst under Xcode
    let filter = log::LevelFilter::Debug;
    if cfg!(target_abi = "macabi") {
        simple_logger::SimpleLogger::new()
            .with_level(filter)
            .env()
            .init()
            .unwrap();
    } else {
        oslog::OsLogger::new(module_path!())
            .level_filter(filter)
            .init()
            .unwrap();
    }

    println!("Hello, world!");
}
