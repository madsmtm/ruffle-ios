use std::cell::{Cell, OnceCell, RefCell};

use block2::{Block, RcBlock};
use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSArray, NSBundle, NSCoder, NSIndexPath, NSInteger,
    NSObjectProtocol, NSString,
};
use objc2_ui_kit::{
    NSIndexPathUIKitAdditions, UIAction, UIButton, UILabel, UIMenu, UIMenuElementState,
    UIMenuOptions, UINavigationItem, UIScrollViewDelegate, UISegmentedControl, UITableView,
    UITableViewCell, UITableViewDataSource, UITableViewDelegate, UITextField, UIViewController,
};
use ruffle_core::{LoadBehavior, PlayerRuntime, StageAlign, StageScaleMode};
use ruffle_frontend_utils::bundle::info::BundleInformation;
use ruffle_frontend_utils::player_options::PlayerOptions;
use ruffle_render::quality::StageQuality;

#[derive(Clone, Copy, Debug)]
enum FormElement {
    Name,
    Source,
    String {
        label: &'static str,
        text: fn(&PlayerOptions) -> Option<String>,
    },
    Select {
        label: &'static str,
        variants: &'static [&'static str],
        enabled_variant: fn(&PlayerOptions) -> Option<&'static str>,
    },
    Bool {
        label: &'static str,
        value: fn(&PlayerOptions) -> Option<bool>,
    },
}

// TODO: Localization
const FORM: &[&[FormElement]] = &[
    // Required
    &[FormElement::Name, FormElement::Source],
    // General options
    &[
        FormElement::String {
            label: "Maximum execution duration (s)",
            text: |options| {
                options
                    .max_execution_duration
                    .map(|duration| duration.as_secs().to_string())
            },
        },
        FormElement::Select {
            label: "Quality",
            variants: &[
                "Low",
                "Medium",
                "High",
                "Best",
                "High (8x8)",
                "High (8x8) Linear",
                "High (16x16)",
                "High (16x16) Linear",
            ],
            enabled_variant: |options| {
                options.quality.map(|quality| match quality {
                    StageQuality::Low => "Low",
                    StageQuality::Medium => "Medium",
                    StageQuality::High => "High",
                    StageQuality::Best => "Best",
                    StageQuality::High8x8 => "High (8x8)",
                    StageQuality::High8x8Linear => "High (8x8) Linear",
                    StageQuality::High16x16 => "High (16x16)",
                    StageQuality::High16x16Linear => "High (16x16) Linear",
                })
            },
        },
        FormElement::String {
            label: "Player version",
            text: |options| options.player_version.map(|version| version.to_string()),
        },
        FormElement::Select {
            label: "Player runtime",
            variants: &["Flash Player", "Adobe AIR"],
            enabled_variant: |options| {
                options.player_runtime.map(|runtime| match runtime {
                    PlayerRuntime::FlashPlayer => "Flash Player",
                    PlayerRuntime::AIR => "Adobe AIR",
                })
            },
        },
        FormElement::String {
            label: "Custom framerate (fps)",
            text: |options| options.frame_rate.map(|rate: f64| rate.to_string()),
        },
    ],
    // Stage Alignment
    &[
        FormElement::Select {
            label: "Alignment",
            variants: &[
                "Center",
                "Top",
                "Bottom",
                "Left",
                "Right",
                "Top-Left",
                "Top-Right",
                "Bottom-Left",
                "Bottom-Right",
            ],
            enabled_variant: |options| {
                const CENTER: StageAlign = StageAlign::empty();
                const TOP_LEFT: StageAlign = StageAlign::TOP.union(StageAlign::LEFT);
                const TOP_RIGHT: StageAlign = StageAlign::TOP.union(StageAlign::RIGHT);
                const BOTTOM_LEFT: StageAlign = StageAlign::BOTTOM.union(StageAlign::LEFT);
                const BOTTOM_RIGHT: StageAlign = StageAlign::BOTTOM.union(StageAlign::RIGHT);
                options.align.map(|align| match align {
                    CENTER => "Center",
                    StageAlign::TOP => "Top",
                    StageAlign::BOTTOM => "Bottom",
                    StageAlign::LEFT => "Left",
                    StageAlign::RIGHT => "Right",
                    TOP_LEFT => "Top-Left",
                    TOP_RIGHT => "Top-Right",
                    BOTTOM_LEFT => "Bottom-Left",
                    BOTTOM_RIGHT => "Bottom-Right",
                    // Fallback
                    _ => "Center",
                })
            },
        },
        FormElement::Bool {
            label: "Force",
            value: |options| options.force_align,
        },
    ],
    // Scale mode
    &[
        FormElement::Select {
            label: "Scale mode",
            variants: &[
                "Center",
                "Top",
                "Bottom",
                "Left",
                "Right",
                "Top-Left",
                "Top-Right",
                "Bottom-Left",
                "Bottom-Right",
            ],
            enabled_variant: |options| {
                options.scale.map(|scale| match scale {
                    StageScaleMode::NoScale => "Unscaled (100%)",
                    StageScaleMode::ShowAll => "Zoom to Fit",
                    StageScaleMode::ExactFit => "Stretch to Fit",
                    StageScaleMode::NoBorder => "Crop to Fit",
                })
            },
        },
        FormElement::Bool {
            label: "Force",
            value: |options| options.force_scale,
        },
    ],
    // Network settings
    &[
        FormElement::String {
            label: "Custom base URL",
            text: |options| options.base.as_ref().map(|url| url.to_string()),
        },
        FormElement::String {
            label: "Spoof SWF URL",
            text: |options| options.spoof_url.as_ref().map(|url| url.to_string()),
        },
        FormElement::String {
            label: "Referer URL",
            text: |options| options.referer.as_ref().map(|url| url.to_string()),
        },
        FormElement::String {
            label: "Cookie",
            text: |options| options.cookie.clone(),
        },
        FormElement::Bool {
            label: "Upgrade HTTP to HTTPS",
            value: |options| options.upgrade_to_https,
        },
        FormElement::Select {
            label: "Load behaviour",
            variants: &["Streaming", "Delayed", "Blocking"],
            enabled_variant: |options| {
                options.load_behavior.map(|behaviour| match behaviour {
                    LoadBehavior::Streaming => "Streaming",
                    LoadBehavior::Delayed => "Delayed",
                    LoadBehavior::Blocking => "Blocking",
                })
            },
        },
        FormElement::Bool {
            label: "Dummy external interface",
            value: |options| options.dummy_external_interface,
        },
    ],
    // Movie parameters are placed at the end
];

// Roughly matches PlayerOptions

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    #[default]
    New,
    Edit,
}

#[derive(Default)]
pub struct Ivars {
    navigation_item: OnceCell<Retained<UINavigationItem>>,
    table_view: OnceCell<Retained<UITableView>>,
    action: Cell<Action>,
    info: RefCell<Option<BundleInformation>>,
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
            unsafe {
                msg_send_id![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil]
            }
        }

        #[method_id(initWithCoder:)]
        fn _init_with_coder(this: Allocated<Self>, coder: &NSCoder) -> Option<Retained<Self>> {
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

        #[method(viewWillAppear:)]
        fn _view_will_appear(&self, animated: bool) {
            self.view_will_appear();
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewWillAppear: animated] };
        }

        #[method(viewDidAppear:)]
        fn _view_did_appear(&self, animated: bool) {
            self.view_did_appear();
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewDidAppear: animated] };
        }
    }

    // Storyboard
    // See storyboard_connections.h
    unsafe impl EditController {
        #[method(setNavigationItem:)]
        fn _set_navigation_item(&self, item: &UINavigationItem) {
            tracing::trace!("edit set navigation item");
            self.ivars()
                .navigation_item
                .set(item.retain())
                .expect("only set navigation item once");
        }

        #[method(setTableView:)]
        fn _set_table_view(&self, table_view: &UITableView) {
            tracing::trace!("edit set table view");
            self.ivars()
                .table_view
                .set(table_view.retain())
                .expect("only set table view once");
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UITableViewDataSource for EditController {
        #[method(tableView:numberOfRowsInSection:)]
        fn tableView_numberOfRowsInSection(
            &self,
            _table_view: &UITableView,
            section: NSInteger,
        ) -> NSInteger {
            if FORM.len() == section as usize {
                let info = self.ivars().info.borrow();
                let options = &info.as_ref().expect("initialized").player;
                options.parameters.len() as NSInteger + 1
            } else {
                FORM[section as usize].len() as NSInteger
            }
        }

        #[method(numberOfSectionsInTableView:)]
        fn numberOfSectionsInTableView(&self, _table_view: &UITableView) -> NSInteger {
            FORM.len() as NSInteger + 1
        }

        #[method_id(tableView:cellForRowAtIndexPath:)]
        fn tableView_cellForRowAtIndexPath(
            &self,
            table_view: &UITableView,
            index_path: &NSIndexPath,
        ) -> Retained<UITableViewCell> {
            self.cell_at_index_path(table_view, index_path)
        }
    }

    unsafe impl UIScrollViewDelegate for EditController {}

    unsafe impl UITableViewDelegate for EditController {}
);

impl EditController {
    pub fn configure(&self, action: Action, info: BundleInformation) {
        self.ivars().action.set(action);
        *self.ivars().info.borrow_mut() = Some(info);
    }

    fn view_did_load(&self) {
        tracing::info!("edit viewDidLoad");
    }

    fn view_will_appear(&self) {
        tracing::info!("edit viewWillAppear:");

        let action = self.ivars().action.get();

        // Configure title bar
        let title = if action == Action::New {
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
    }

    fn view_did_appear(&self) {
        tracing::info!("edit viewDidAppear:");

        // Do the same thing as UITableViewController, flash the scroll bar
        let table = self.ivars().table_view.get().expect("table view");
        unsafe { table.flashScrollIndicators() };
    }

    fn cell_at_index_path(
        &self,
        table_view: &UITableView,
        index_path: &NSIndexPath,
    ) -> Retained<UITableViewCell> {
        let mtm = MainThreadMarker::from(self);
        let info = self.ivars().info.borrow();
        let info = info.as_ref().expect("initialized info");
        let options = &info.player;
        unsafe {
            let section = index_path.section() as usize;
            let row = index_path.row() as usize;
            if FORM.len() == section {
                if options.parameters.len() == row {
                    return table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                        ns_string!("movie-parameter-add"),
                        index_path,
                    );
                }

                let (param, value) = &options.parameters[row];
                let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                    ns_string!("movie-parameter"),
                    index_path,
                );
                let subviews = cell.contentView().subviews();
                let ui_param = Retained::cast::<UITextField>(subviews.objectAtIndex(1));
                ui_param.setText(Some(&NSString::from_str(param)));
                let ui_value = Retained::cast::<UITextField>(subviews.objectAtIndex(2));
                ui_value.setText(Some(&NSString::from_str(value)));

                return cell;
            }

            match FORM[section][row] {
                FormElement::Name => {
                    let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                        ns_string!("root-name"),
                        index_path,
                    );
                    let input = Retained::cast::<UITextField>(
                        cell.contentView().subviews().objectAtIndex(0),
                    );
                    input.setText(Some(&NSString::from_str(&info.name)));
                    cell
                }
                // TODO
                FormElement::Source => {
                    let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                        ns_string!("root-name"),
                        index_path,
                    );
                    let input = Retained::cast::<UITextField>(
                        cell.contentView().subviews().objectAtIndex(0),
                    );
                    input.setText(Some(&NSString::from_str(&info.url.to_string())));
                    cell
                }
                FormElement::String { label, text } => {
                    let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                        ns_string!("string"),
                        index_path,
                    );
                    let subviews = cell.contentView().subviews();

                    let ui_label = Retained::cast::<UILabel>(subviews.objectAtIndex(0));
                    ui_label.setText(Some(&NSString::from_str(label)));

                    let input = Retained::cast::<UITextField>(subviews.objectAtIndex(1));
                    input.setText(text(&options).map(|s| NSString::from_str(&s)).as_deref());
                    cell
                }
                FormElement::Select {
                    label,
                    variants,
                    enabled_variant,
                } => {
                    let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                        ns_string!("select"),
                        index_path,
                    );
                    let subviews = cell.contentView().subviews();

                    let ui_label = Retained::cast::<UILabel>(subviews.objectAtIndex(0));
                    ui_label.setText(Some(&NSString::from_str(label)));

                    // Set menu
                    let enabled_variant = enabled_variant(&options);
                    let button = Retained::cast::<UIButton>(subviews.objectAtIndex(1));
                    // We have to use UIAction here, UICommand seems to be broken
                    let block = RcBlock::new(|_| {});
                    let block_ptr: *const Block<_> = &*block;
                    let default_item = UIAction::actionWithHandler(block_ptr.cast_mut(), mtm);
                    default_item.setTitle(ns_string!("Default"));
                    if enabled_variant.is_none() {
                        default_item.setState(UIMenuElementState::On);
                    }

                    let children: Retained<NSArray<_>> = variants
                        .iter()
                        .map(|title| {
                            let cmd = UIAction::actionWithHandler(block_ptr.cast_mut(), mtm);
                            cmd.setTitle(&NSString::from_str(title));
                            if enabled_variant == Some(title) {
                                cmd.setState(UIMenuElementState::On);
                            }
                            Retained::into_super(cmd)
                        })
                        .collect();
                    button.setMenu(Some(
                        &UIMenu::menuWithTitle_image_identifier_options_children(
                            ns_string!(""),
                            None,
                            None,
                            UIMenuOptions::SingleSelection,
                            &NSArray::from_slice(&[
                                &**default_item,
                                &*UIMenu::menuWithTitle_image_identifier_options_children(
                                    ns_string!(""),
                                    None,
                                    None,
                                    UIMenuOptions::DisplayInline | UIMenuOptions::SingleSelection,
                                    &children,
                                    mtm,
                                ),
                            ]),
                            mtm,
                        ),
                    ));
                    cell
                }
                FormElement::Bool { label, value } => {
                    let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                        ns_string!("bool"),
                        index_path,
                    );
                    let subviews = cell.contentView().subviews();

                    let ui_label = Retained::cast::<UILabel>(subviews.objectAtIndex(0));
                    ui_label.setText(Some(&NSString::from_str(label)));

                    let control = Retained::cast::<UISegmentedControl>(subviews.objectAtIndex(1));
                    control.setSelectedSegmentIndex(match value(&options) {
                        None => 0,
                        Some(false) => 1,
                        Some(true) => 2,
                    });
                    cell
                }
            }
        }
    }
}
