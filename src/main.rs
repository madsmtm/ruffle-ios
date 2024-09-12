use std::ptr::NonNull;

use delegate::Delegate;
use objc2::ClassType;
use objc2_foundation::{MainThreadMarker, NSStringFromClass};
use objc2_ui_kit::UIApplicationMain;

mod delegate;

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

    let _ = MainThreadMarker::new().unwrap();
    unsafe {
        UIApplicationMain(
            *libc::_NSGetArgc(),
            NonNull::new(*libc::_NSGetArgv()).unwrap(),
            None,
            Some(NSStringFromClass(Delegate::class()).as_ref()),
        );
    }
}
