use objc2::rc::Retained;
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{CGRect, CGSize, MainThreadMarker, NSObjectProtocol};
use objc2_ui_kit::UIView;

pub struct Ivars {}

declare_class!(
    pub struct PlayerView;

    unsafe impl ClassType for PlayerView {
        type Super = UIView;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "PlayerView";
    }

    impl DeclaredClass for PlayerView {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for PlayerView {}

    unsafe impl PlayerView {
        #[method(drawRect:)]
        fn draw_rect(&self, _rect: CGRect) {
            tracing::debug!("triggered `drawRect:`");
            // No need to call super, it does nothing on `UIView`.
        }

        // `layoutSubviews` is the recommended way to listen for changes to
        // the view's frame. Also tracks changes to the backing scale factor.
        #[method(layoutSubviews)]
        fn layout_subviews(&self) {
            let new_size = scaled_view_frame(self);
            tracing::debug!("triggered `layoutSubviews`, new_size: {:?}", new_size);
            // Calling super here is not really necessary, as we have no
            // subviews, but we do it anyway just to make sure.
            let _: () = unsafe { objc2::msg_send![super(self), layoutSubviews] };
        }
    }
);

impl PlayerView {
    pub fn new(mtm: MainThreadMarker, frame_rect: CGRect) -> Retained<Self> {
        // Create view
        let view = mtm.alloc().set_ivars(Ivars {});
        let view: Retained<Self> = unsafe { msg_send_id![super(view), initWithFrame: frame_rect] };

        // Ensure that the view calls `drawRect:` after being resized
        unsafe {
            view.setContentMode(objc2_ui_kit::UIViewContentMode::Redraw);
        }

        view
    }
}

fn scaled_view_frame(view: &UIView) -> CGSize {
    let size = view.frame().size;
    let scale_factor = view.contentScaleFactor();
    CGSize {
        width: size.width * scale_factor,
        height: size.height * scale_factor,
    }
}
