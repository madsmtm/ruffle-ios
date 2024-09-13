use std::cell::OnceCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

use objc2::rc::Retained;
use objc2::{declare_class, msg_send, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{CGRect, MainThreadMarker, NSObjectProtocol};
use objc2_ui_kit::UIViewController;
use ruffle_core::config::Letterbox;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder};
use ruffle_frontend_utils::backends::executor::{AsyncExecutor, PollRequester};
use ruffle_frontend_utils::backends::navigator::{ExternalNavigatorBackend, NavigatorInterface};
use ruffle_frontend_utils::content::PlayingContent;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use url::Url;

use crate::player_view::PlayerView;

#[derive(Clone)]
pub struct EventSender(Rc<OnceCell<Arc<AsyncExecutor<EventSender>>>>);

impl PollRequester for EventSender {
    fn request_poll(&self) {
        eprintln!("request_poll, main: {}", MainThreadMarker::new().is_some());
        self.0.get().expect("initialized").poll_all();
    }
}

pub struct Ivars {
    movie_url: String,
    player: OnceCell<Arc<Mutex<Player>>>,
    executor: OnceCell<Arc<AsyncExecutor<EventSender>>>,
}

#[derive(Clone)]
struct Navigator;

impl NavigatorInterface for Navigator {
    fn navigate_to_website(&self, _url: Url, _ask: bool) {}

    fn open_file(&self, path: &Path) -> io::Result<File> {
        File::open(path)
    }

    async fn confirm_socket(&self, _host: &str, _port: u16) -> bool {
        true
    }
}

declare_class!(
    #[derive(Debug)]
    pub struct PlayerController;

    unsafe impl ClassType for PlayerController {
        type Super = UIViewController;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "PlayerController";
    }

    impl DeclaredClass for PlayerController {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for PlayerController {}

    unsafe impl PlayerController {
        #[method(loadView)]
        fn _load_view(&self) {
            self.load_view();
        }

        #[method(viewDidLoad)]
        fn _view_did_load(&self) {
            self.view_did_load();
        }

        #[method(viewIsAppearing:)]
        fn _view_is_appearing(&self, animated: bool) {
            self.view_is_appearing(animated);
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewIsAppearing: animated] };
        }

        #[method(viewWillDisappear:)]
        fn _view_will_disappear(&self, animated: bool) {
            self.view_will_disappear(animated);
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewWillDisappear: animated] };
        }

        #[method(viewDidDisappear:)]
        fn _view_did_disappear(&self, animated: bool) {
            self.view_did_disappear(animated);
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewDidDisappear: animated] };
        }
    }
);

impl PlayerController {
    pub fn new(mtm: MainThreadMarker, movie_url: String) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(Ivars {
            movie_url,
            player: OnceCell::new(),
            executor: OnceCell::new(),
        });
        unsafe { msg_send_id![super(this), init] }
    }

    fn load_view(&self) {
        tracing::info!("loadView");
        let mtm = MainThreadMarker::from(self);
        let view = PlayerView::new(mtm, CGRect::default());
        self.setView(Some(&view));
    }

    fn view_did_load(&self) {
        tracing::info!("viewDidLoad");

        // TODO: Specify safe area somehow
        let view = self.view();
        let layer = view.layer();
        let layer_ptr = Retained::as_ptr(&layer).cast_mut().cast();
        let renderer = unsafe {
            WgpuRenderBackend::for_window_unsafe(
                wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr),
                (1, 1),
                wgpu::Backends::METAL,
                wgpu::PowerPreference::HighPerformance,
                None,
            )
            .expect("creating renderer")
        };

        let sender = EventSender(Rc::new(OnceCell::new()));
        let (executor, future_spawner) = AsyncExecutor::new(sender.clone());
        sender
            .0
            .set(executor.clone())
            .unwrap_or_else(|_| panic!("init once"));

        let movie_url = Url::parse("file://movie.swf").unwrap();
        let navigator = ExternalNavigatorBackend::new(
            movie_url.clone(),
            None,
            None,
            future_spawner,
            None,
            true,
            ruffle_core::backend::navigator::OpenURLMode::Allow,
            Default::default(),
            ruffle_core::backend::navigator::SocketMode::Allow,
            Rc::new(PlayingContent::DirectFile(movie_url)),
            Navigator,
        );

        // Temporary until we figure out actual loading
        let movie =
            SwfMovie::from_path(&self.ivars().movie_url, None).expect("failed loading movie");

        let player = PlayerBuilder::new()
            .with_renderer(renderer)
            .with_navigator(navigator)
            .with_movie(movie)
            .build();

        let mut player_lock = player.lock().unwrap();
        // player_lock.fetch_root_movie(
        //     self.ivars().movie_url.clone(),
        //     vec![],
        //     Box::new(|metadata| {
        //         eprintln!("got movie: {:?}", metadata);
        //     }),
        // );
        player_lock.set_letterbox(Letterbox::On);
        drop(player_lock);

        view.set_player(player.clone());
        self.ivars()
            .player
            .set(player)
            .unwrap_or_else(|_| panic!("viewDidLoad once"));
        self.ivars()
            .executor
            .set(executor)
            .unwrap_or_else(|_| panic!("viewDidLoad once"));

        // last_frame_time = Instant::now();
        // next_frame_time = Some(Instant::now());
    }

    fn view_is_appearing(&self, _animated: bool) {
        tracing::info!("viewIsAppearing:");

        self.player_lock().set_is_playing(true);
    }

    fn view_will_disappear(&self, _animated: bool) {
        tracing::info!("viewWillDisappear:");

        self.player_lock().set_is_playing(false);
    }

    fn view_did_disappear(&self, _animated: bool) {
        tracing::info!("viewDidDisappear:");

        self.player_lock().flush_shared_objects();
    }

    fn view(&self) -> Retained<PlayerView> {
        let view = (**self).view().expect("controller loads view");
        assert!(
            view.isKindOfClass(PlayerView::class()),
            "must have correct view type"
        );
        // SAFETY: Just checked that the view is of type `PlayerView`
        unsafe { Retained::cast(view) }
    }

    #[track_caller]
    fn player_lock(&self) -> MutexGuard<'_, Player> {
        self.ivars()
            .player
            .get()
            .expect("player initialized")
            .lock()
            .expect("player lock")
    }
}
