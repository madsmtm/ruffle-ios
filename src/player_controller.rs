use objc2::rc::Retained;
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{CGRect, MainThreadMarker, NSObjectProtocol};
use objc2_ui_kit::UIViewController;

use crate::player_view::PlayerView;

declare_class!(
    #[derive(Debug)]
    pub struct PlayerController;

    unsafe impl ClassType for PlayerController {
        type Super = UIViewController;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "PlayerController";
    }

    impl DeclaredClass for PlayerController {
        type Ivars = ();
    }

    unsafe impl NSObjectProtocol for PlayerController {}
);

impl PlayerController {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(());
        let this: Retained<Self> = unsafe { msg_send_id![super(this), init] };
        this.setView(Some(&PlayerView::new(mtm, CGRect::default())));
        this
    }
}
