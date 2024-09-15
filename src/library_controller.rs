use std::cell::OnceCell;

use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{ns_string, NSBundle, NSCoder, NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{NSDataAsset, UIBarButtonItem, UIStoryboardSegue, UITableViewController};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;
use ruffle_frontend_utils::backends::audio::CpalAudioBackend;

use crate::PlayerView;

#[derive(Default)]
pub struct Ivars {
    logo_view: OnceCell<Retained<PlayerView>>,
}

declare_class!(
    #[derive(Debug)]
    pub struct LibraryController;

    unsafe impl ClassType for LibraryController {
        type Super = UITableViewController;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "LibraryController";
    }

    impl DeclaredClass for LibraryController {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for LibraryController {}

    unsafe impl LibraryController {
        #[method_id(initWithNibName:bundle:)]
        fn _init_with_nib_name_bundle(
            this: Allocated<Self>,
            nib_name_or_nil: Option<&NSString>,
            nib_bundle_or_nil: Option<&NSBundle>,
        ) -> Retained<Self> {
            tracing::info!("library init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil] }
        }

        #[method_id(initWithCoder:)]
        fn _init_with_coder(
            this: Allocated<Self>,
            coder: &NSCoder,
        ) -> Option<Retained<Self>> {
            tracing::info!("library init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithCoder: coder] }
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

    // Storyboard
    // See storyboard_connections.h
    unsafe impl LibraryController {
        #[method(setLogoView:)]
        fn _set_logo_view(&self, view: Option<&PlayerView>) {
            tracing::trace!("library set logo view");
            let view = view.expect("logo view not null");
            assert!(view.isKindOfClass(PlayerView::class()), "logo view not a PlayerView");
            self.ivars().logo_view.set(view.retain()).expect("only set logo view once");
        }

        #[method(toggleEditing:)]
        fn _toggle_editing(&self, sender: Option<&NSObject>) {
            tracing::trace!("library toggle editing");
            let button = sender.expect("edit button");
            assert!(button.isKindOfClass(UIBarButtonItem::class()), "edit button not UIBarButtonItem");
            let button = unsafe { &*(button as *const NSObject as *const UIBarButtonItem) };
            self.toggle_editing(button);
        }

        #[method(editingClosed:)]
        #[allow(deprecated)]
        fn _editing_closed(&self, _segue: &UIStoryboardSegue) {}
    }
);

impl LibraryController {
    fn logo_view(&self) -> &PlayerView {
        self.ivars().logo_view.get().expect("logo view initialized")
    }

    fn view_did_load(&self) {
        tracing::info!("library viewDidLoad");

        self.setup_logo();
    }

    fn setup_logo(&self) {
        let view = self.logo_view();
        let asset =
            unsafe { NSDataAsset::initWithName(NSDataAsset::alloc(), ns_string!("logo-anim")) }
                .expect("asset store should contain logo-anim");
        let data = unsafe { asset.data() };
        let movie = SwfMovie::from_data(data.bytes(), "file://logo-anim.swf".into(), None)
            .expect("loading movie");

        let renderer = view.create_renderer();

        let mut builder = PlayerBuilder::new()
            .with_renderer(renderer)
            .with_movie(movie);

        match CpalAudioBackend::new(None) {
            Ok(audio) => builder = builder.with_audio(audio),
            Err(e) => tracing::error!("Unable to create audio device: {e}"),
        }

        view.set_player(builder.build());
        // HACK: Skip first frame to avoid a flicker on startup
        // FIXME: This probably indicates a bug in our timing code?
        view.player_lock().run_frame();
    }

    fn view_is_appearing(&self, _animated: bool) {
        tracing::info!("library viewIsAppearing:");

        self.logo_view().start();
    }

    fn view_will_disappear(&self, _animated: bool) {
        tracing::info!("library viewWillDisappear:");

        self.logo_view().stop();
    }

    fn view_did_disappear(&self, _animated: bool) {
        tracing::info!("library viewDidDisappear:");

        self.logo_view().player_lock().flush_shared_objects();
    }

    fn toggle_editing(&self, button: &UIBarButtonItem) {
        unsafe {
            let table_view = self.tableView().expect("has table view");
            let is_editing = !table_view.isEditing();
            table_view.setEditing_animated(is_editing, true);
            button.setTitle(Some(if is_editing {
                ns_string!("Done")
            } else {
                ns_string!("Edit")
            }));
        }
    }
}
