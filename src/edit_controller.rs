use std::cell::{Cell, OnceCell};

use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{ns_string, NSBundle, NSCoder, NSObjectProtocol, NSString};
use objc2_ui_kit::{UINavigationItem, UITableView, UIViewController};
use ruffle_frontend_utils::player_options::PlayerOptions;

#[derive(Debug, Default)]
pub enum Action {
    #[default]
    New,
    Edit(PlayerOptions),
}

#[derive(Default)]
pub struct Ivars {
    navigation_item: OnceCell<Retained<UINavigationItem>>,
    table_view: OnceCell<Retained<UITableView>>,
    action: Cell<Action>,
    current_data: PlayerOptions,
}

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

        #[method(viewDidLoad)]
        fn _view_did_load(&self) {
            // Xcode template calls super at the beginning
            let _: () = unsafe { msg_send![super(self), viewDidLoad] };
            self.view_did_load();
        }

        #[method(viewIsAppearing:)]
        fn _view_is_appearing(&self, animated: bool) {
            self.view_is_appearing();
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewIsAppearing: animated] };
        }
    }

    // Storyboard
    // See storyboard_connections.h
    unsafe impl EditController {
        #[method(setNavigationItem:)]
        fn _set_navigation_item(&self, item: &UINavigationItem) {
            tracing::trace!("edit set navigation item");
            self.ivars().navigation_item.set(item.retain()).expect("only set navigation item once");
        }

        #[method(setTableView:)]
        fn _set_table_view(&self, table_view: &UITableView) {
            tracing::trace!("edit set table view");
            self.ivars().table_view.set(table_view.retain()).expect("only set table view once");
        }
    }
);

impl EditController {
    pub fn set_action(&self, action: Action) {
        self.ivars().action.set(action);
    }

    fn view_did_load(&self) {
        tracing::info!("edit viewDidLoad");

        // Initialize table
        let table = self.ivars().table_view.get().expect("table view");
    }

    fn view_is_appearing(&self) {
        tracing::info!("edit viewIsAppearing:");

        let action = self.ivars().action.take();

        // Configure title bar
        let title = if matches!(action, Action::New) {
            ns_string!("Add SWF")
        } else {
            ns_string!("Edit SWF")
        };
        unsafe {
            self.ivars()
                .navigation_item
                .get()
                .expect("navigation item set")
                .setTitle(Some(title));
        }

        let options = match action {
            Action::New => PlayerOptions::default(),
            Action::Edit(options) => options,
        };

        // Set data in table
        let table = self.ivars().table_view.get().expect("table view");
    }

    fn get_data(&self) -> PlayerOptions {
        let table = self.ivars().table_view.get().expect("table view");
        todo!()
    }
}
