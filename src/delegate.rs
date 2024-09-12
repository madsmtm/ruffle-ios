use std::cell::OnceCell;

use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use objc2_ui_kit::{UIApplication, UIApplicationDelegate, UIScreen, UIViewController, UIWindow};

declare_class!(
    #[derive(Debug)]
    pub struct ViewController;

    unsafe impl ClassType for ViewController {
        type Super = UIViewController;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "ViewController";
    }

    impl DeclaredClass for ViewController {
        type Ivars = ();
    }

    unsafe impl NSObjectProtocol for ViewController {}
);

impl ViewController {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(());
        unsafe { msg_send_id![super(this), init] }
    }
}

#[derive(Debug)]
pub struct Ivars {
    window: OnceCell<Retained<UIWindow>>,
}

declare_class!(
    #[derive(Debug)]
    pub struct Delegate;

    unsafe impl ClassType for Delegate {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "Delegate";
    }

    impl DeclaredClass for Delegate {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl Delegate {
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(Ivars {
                window: OnceCell::new(),
            });
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl UIApplicationDelegate for Delegate {
        #[method(applicationDidFinishLaunching:)]
        fn did_finish_launching(&self, _application: &UIApplication) {
            tracing::info!("applicationDidFinishLaunching:");
            self.setup();
        }
    }
);

impl Delegate {
    fn setup(&self) {
        let mtm = MainThreadMarker::from(self);

        #[allow(deprecated)] // Unsure how else we should do this when setting up?
        let frame = UIScreen::mainScreen(mtm).bounds();

        let window = unsafe { UIWindow::initWithFrame(mtm.alloc(), frame) };

        let view_controller = ViewController::new(mtm);
        window.setRootViewController(Some(&view_controller));

        window.makeKeyAndVisible();

        self.ivars()
            .window
            .set(window)
            .expect("can only initialize once");
    }
}
