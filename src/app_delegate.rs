use std::cell::OnceCell;

use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_ui_kit::{UIApplication, UIApplicationDelegate, UIWindow};

#[derive(Debug)]
pub struct Ivars {
    _window: OnceCell<Retained<UIWindow>>,
}

declare_class!(
    #[derive(Debug)]
    pub struct AppDelegate;

    unsafe impl ClassType for AppDelegate {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "AppDelegate";
    }

    impl DeclaredClass for AppDelegate {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl AppDelegate {
        // Called by UIKitApplicationMain
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(Ivars {
                _window: OnceCell::new(),
            });
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl UIApplicationDelegate for AppDelegate {
        #[method(applicationDidFinishLaunching:)]
        fn did_finish_launching(&self, _application: &UIApplication) {
            tracing::info!("applicationDidFinishLaunching:");
        }
    }
);
