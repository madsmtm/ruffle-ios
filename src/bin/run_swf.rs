//! Run an SWF without setting up navigation, a data model and everything.
use std::cell::OnceCell;

use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use objc2_ui_kit::{UIApplication, UIApplicationDelegate, UIScreen, UIWindow};

use ruffle_ios::{init_logging, launch, PlayerController};

#[derive(Debug)]
pub struct Ivars {
    window: OnceCell<Retained<UIWindow>>,
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
                window: OnceCell::new(),
            });
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl UIApplicationDelegate for AppDelegate {
        #[method(applicationDidFinishLaunching:)]
        fn did_finish_launching(&self, _application: &UIApplication) {
            tracing::info!("applicationDidFinishLaunching:");
            self.setup();
        }
    }
);

impl AppDelegate {
    fn setup(&self) {
        let movie_url = std::env::args().skip(1).next();
        let movie_url = movie_url.expect("must provide a path or URL to an SWF to run");
        let mtm = MainThreadMarker::from(self);

        #[allow(deprecated)] // Unsure how else we should do this when setting up?
        let frame = UIScreen::mainScreen(mtm).bounds();

        let window = unsafe { UIWindow::initWithFrame(mtm.alloc(), frame) };

        let view_controller = PlayerController::new(mtm, movie_url);
        window.setRootViewController(Some(&view_controller));

        window.makeKeyAndVisible();

        self.ivars()
            .window
            .set(window)
            .expect("can only initialize once");
    }
}

#[tokio::main]
async fn main() {
    init_logging();
    launch(None, Some(AppDelegate::class()));
}
