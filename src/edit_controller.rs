use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{NSBundle, NSCoder, NSObjectProtocol, NSString};
#[allow(deprecated)]
use objc2_ui_kit::UIViewController;

#[derive(Default)]
pub struct Ivars {}

declare_class!(
    #[derive(Debug)]
    pub struct EditController;

    unsafe impl ClassType for EditController {
        type Super = UIViewController;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "EditController";
    }

    impl DeclaredClass for EditController {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for EditController {}

    unsafe impl EditController {
        #[method_id(initWithNibName:bundle:)]
        fn _init_with_nib_name_bundle(
            this: Allocated<Self>,
            nib_name_or_nil: Option<&NSString>,
            nib_bundle_or_nil: Option<&NSBundle>,
        ) -> Retained<Self> {
            tracing::info!("edit init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil] }
        }

        #[method_id(initWithCoder:)]
        fn _init_with_coder(
            this: Allocated<Self>,
            coder: &NSCoder,
        ) -> Option<Retained<Self>> {
            tracing::info!("edit init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithCoder: coder] }
        }
    }
);
