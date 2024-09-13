use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{ns_string, CGRect, NSObjectProtocol};
use objc2_ui_kit::{NSDataAsset, UIColor};
use ruffle_core::config::Letterbox;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;

use crate::player_view::PlayerView;

#[derive(Debug)]
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
            Self::init_with_frame(this, frame)
        }
    }
);

impl Drop for LogoView {
    fn drop(&mut self) {
        // TODO: Do we really need to do this?
        let mut player_lock = self.player_lock();
        player_lock.set_is_playing(false);
        player_lock.flush_shared_objects();
    }
}

impl LogoView {
    fn init_with_frame(this: Allocated<Self>, frame: CGRect) -> Retained<Self> {
        let asset =
            unsafe { NSDataAsset::initWithName(NSDataAsset::alloc(), ns_string!("logo-anim")) }
                .expect("asset store should contain logo-anim");
        let data = unsafe { asset.data() };
        let movie = SwfMovie::from_data(data.bytes(), "file://logo-anim.swf".into(), None)
            .expect("loading movie");

        let this = this.set_ivars(Ivars {});
        let this: Retained<Self> = unsafe { msg_send_id![super(this), initWithFrame: frame] };

        let renderer = this.create_renderer();

        let player = PlayerBuilder::new()
            .with_renderer(renderer)
            .with_movie(movie)
            .build();

        let mut player_lock = player.lock().unwrap();
        player_lock.set_letterbox(Letterbox::On);
        player_lock.set_is_playing(true);
        drop(player_lock);

        let bg_color =
            unsafe { UIColor::colorNamed(ns_string!("ruffle-blue")) }.expect("ruffle blue");
        this.setBackgroundColor(Some(&bg_color));

        this.set_player(player);

        this
    }
}
