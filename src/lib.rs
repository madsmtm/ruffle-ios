use std::ptr::NonNull;

use objc2::runtime::AnyClass;
use objc2_foundation::{MainThreadMarker, NSStringFromClass};
use objc2_ui_kit::UIApplicationMain;

mod app_delegate;
mod player_controller;
mod player_view;

pub use self::app_delegate::AppDelegate;
pub use self::player_controller::PlayerController;
pub use self::player_view::PlayerView;

pub fn init_logging() {
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
}

pub fn launch(app_class: Option<&AnyClass>, delegate_class: Option<&AnyClass>) {
    let _ = MainThreadMarker::new().unwrap();
    unsafe {
        UIApplicationMain(
            *libc::_NSGetArgc(),
            NonNull::new(*libc::_NSGetArgv()).unwrap(),
            app_class.map(|cls| NSStringFromClass(cls).as_ref()),
            delegate_class.map(|cls| NSStringFromClass(cls).as_ref()),
        );
    }
}
