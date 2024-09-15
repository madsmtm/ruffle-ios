use std::ptr::NonNull;

use objc2::runtime::AnyClass;
use objc2::ClassType;
use objc2_foundation::{MainThreadMarker, NSStringFromClass};
use objc2_ui_kit::UIApplicationMain;

mod app_delegate;
mod library_controller;
mod player_controller;
mod player_view;
mod scene_delegate;

pub use self::app_delegate::AppDelegate;
pub use self::player_controller::PlayerController;
pub use self::player_view::PlayerView;

pub fn init_logging() {
    // Emit logging to either OSLog or stderr, depending on if using Mac
    // Catalyst or native.
    // TODO: If running Mac Catalyst under Xcode
    let filter = log::LevelFilter::Info;
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
    // These classes are loaded from a storyboard,
    // and hence need to be initialized first.
    let _ = player_view::PlayerView::class();
    let _ = scene_delegate::SceneDelegate::class();
    let _ = player_controller::PlayerController::class();
    let _ = library_controller::LibraryController::class();

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
