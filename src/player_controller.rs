use std::cell::OnceCell;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{io, ptr};

use objc2::rc::{Allocated, Retained};
use objc2::runtime::AnyObject;
use objc2::{declare_class, msg_send, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{
    ns_string, CGPoint, CGRect, CGSize, MainThreadMarker, NSBundle, NSCoder, NSObjectProtocol,
    NSString,
};
use objc2_ui_kit::{NSDataAsset, UIViewController};
use ruffle_core::config::Letterbox;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder};
use ruffle_frontend_utils::backends::audio::CpalAudioBackend;
use ruffle_frontend_utils::backends::executor::{AsyncExecutor, PollRequester};
use ruffle_frontend_utils::backends::navigator::{ExternalNavigatorBackend, NavigatorInterface};
use ruffle_frontend_utils::content::PlayingContent;
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

#[derive(Default)]
pub struct Ivars {
    movie_path: Option<String>,
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
        #[method_id(initWithNibName:bundle:)]
        fn _init_with_nib_name_bundle(
            this: Allocated<Self>,
            nib_name_or_nil: Option<&NSString>,
            nib_bundle_or_nil: Option<&NSBundle>,
        ) -> Retained<Self> {
            tracing::info!("player controller init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil] }
        }

        #[method_id(initWithCoder:)]
        fn _init_with_coder(
            this: Allocated<Self>,
            coder: &NSCoder,
        ) -> Option<Retained<Self>> {
            tracing::info!("player controller init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithCoder: coder] }
        }

        #[method(loadView)]
        fn _load_view(&self) {
            self.load_view();
        }

        #[method(viewDidLoad)]
        fn _view_did_load(&self) {
            // Xcode template calls super at the beginning
            let _: () = unsafe { msg_send![super(self), viewDidLoad] };
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

    // UIResponder
    #[allow(non_snake_case)]
    unsafe impl PlayerController {
        #[method(canBecomeFirstResponder)]
        fn canBecomeFirstResponder(&self) -> bool {
            true
        }

        #[method(becomeFirstResponder)]
        fn becomeFirstResponder(&self) -> bool {
            tracing::info!("player controller becomeFirstResponder");
            unsafe { self.view().becomeFirstResponder() };
            true
        }

        #[method(canResignFirstResponder)]
        fn canResignFirstResponder(&self) -> bool {
            true
        }

        #[method(resignFirstResponder)]
        fn resignFirstResponder(&self) -> bool {
            tracing::info!("controller resignFirstResponder");
            true
        }
    }
);

impl PlayerController {
    pub fn new(mtm: MainThreadMarker, movie_path: String) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(Ivars {
            movie_path: Some(movie_path),
            player: OnceCell::new(),
            executor: OnceCell::new(),
        });
        let nil = ptr::null::<AnyObject>();
        unsafe { msg_send_id![super(this), initWithNibName: nil, bundle: nil] }
    }

    fn load_view(&self) {
        tracing::info!("player loadView");
        let mtm = MainThreadMarker::from(self);
        let view = PlayerView::initWithFrame(
            mtm.alloc(),
            CGRect::new(CGPoint::ZERO, CGSize::new(1.0, 1.0)),
        );
        self.setView(Some(&view));
    }

    fn view_did_load(&self) {
        tracing::info!("player viewDidLoad");

        // TODO: Specify safe area somehow
        let view = self.view();
        let renderer = view.create_renderer();

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

        let mut builder = PlayerBuilder::new()
            .with_renderer(renderer)
            .with_navigator(navigator);

        // Temporary until we figure out actual loading
        let movie = if let Some(path) = self.ivars().movie_path.as_deref() {
            SwfMovie::from_path(path, None).expect("failed loading movie")
        } else {
            let asset =
                unsafe { NSDataAsset::initWithName(NSDataAsset::alloc(), ns_string!("logo-anim")) }
                    .expect("asset store should contain logo-anim");
            let data = unsafe { asset.data() };
            SwfMovie::from_data(data.bytes(), "file://logo-anim.swf".into(), None)
                .expect("loading movie")
        };
        builder = builder.with_movie(movie);

        match CpalAudioBackend::new(None) {
            Ok(audio) => builder = builder.with_audio(audio),
            Err(e) => tracing::error!("Unable to create audio device: {e}"),
        }

        let player = builder.build();

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
    }

    fn view_is_appearing(&self, _animated: bool) {
        tracing::info!("player viewIsAppearing:");

        self.view().start();
    }

    fn view_will_disappear(&self, _animated: bool) {
        tracing::info!("player viewWillDisappear:");

        self.view().stop();
    }

    fn view_did_disappear(&self, _animated: bool) {
        tracing::info!("player viewDidDisappear:");

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
