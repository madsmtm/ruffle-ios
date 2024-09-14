use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{ns_string, CGRect, NSCoder, NSObjectProtocol};
use objc2_ui_kit::NSDataAsset;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;

use crate::player_view::PlayerView;

#[derive(Debug, Default)]
pub struct Ivars {}

declare_class!(
    pub struct LogoView;

    unsafe impl ClassType for LogoView {
        type Super = PlayerView;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "LogoView";
    }

    impl DeclaredClass for LogoView {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for LogoView {}

    unsafe impl LogoView {
        #[method_id(initWithFrame:)]
        fn _init_with_frame(this: Allocated<Self>, frame: CGRect) -> Retained<Self> {
            let this = this.set_ivars(Ivars::default());
            let this: Retained<Self> = unsafe { msg_send_id![super(this), initWithFrame: frame] };
            this.init();
            this
        }

        #[method_id(initWithCoder:)]
        fn _init_with_coder(this: Allocated<Self>, coder: &NSCoder) -> Retained<Self> {
            let this = this.set_ivars(Ivars::default());
            let this: Retained<Self> = unsafe { msg_send_id![super(this), initWithCoder: coder] };
            this.init();
            this
        }
    }
);

impl Drop for LogoView {
    fn drop(&mut self) {
        self.stop();
        // TODO: Do we really need to do this?
        let mut player_lock = self.player_lock();
        player_lock.flush_shared_objects();
    }
}

impl LogoView {
    fn init(&self) {
        let asset =
            unsafe { NSDataAsset::initWithName(NSDataAsset::alloc(), ns_string!("logo-anim")) }
                .expect("asset store should contain logo-anim");
        let data = unsafe { asset.data() };
        let movie = SwfMovie::from_data(data.bytes(), "file://logo-anim.swf".into(), None)
            .expect("loading movie");

        let renderer = self.create_renderer();

        let player = PlayerBuilder::new()
            .with_renderer(renderer)
            .with_movie(movie)
            .build();

        self.set_player(player);
        // HACK: Skip first frame to avoid a flicker on startup
        // FIXME: This probably indicates a bug in our timing code?
        self.player_lock().run_frame();
        self.start();
        // TODO: Stop the logo view when we change screens
    }
}
