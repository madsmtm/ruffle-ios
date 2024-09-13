use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{ns_string, MainThreadMarker, NSObject, NSObjectProtocol, NSSet};
use objc2_ui_kit::{
    UIApplication, UIApplicationDelegate, UISceneConfiguration, UISceneConnectionOptions,
    UISceneSession,
};

#[derive(Debug)]
pub struct Ivars {}

declare_class!(
    #[derive(Debug)]
    pub struct AppDelegate;

    unsafe impl ClassType for AppDelegate {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "AppDelegate";
    }

    impl DeclaredClass for AppDelegate {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl AppDelegate {
        // Called by UIKitApplicationMain
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(Ivars {});
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl UIApplicationDelegate for AppDelegate {
        #[method(applicationDidFinishLaunching:)]
        fn did_finish_launching(&self, _application: &UIApplication) {
            tracing::info!("applicationDidFinishLaunching:");
        }

        #[method_id(application:configurationForConnectingSceneSession:options:)]
        fn _application_configuration_for_connecting_scene_session_options(
            &self,
            _application: &UIApplication,
            connecting_scene_session: &UISceneSession,
            _options: &UISceneConnectionOptions,
        ) -> Retained<UISceneConfiguration> {
            tracing::info!("application:configurationForConnectingSceneSession:options:");
            // Called when a new scene session is being created.
            // Use this method to select a configuration to create the new scene with.
            let mtm = MainThreadMarker::from(self);
            unsafe {
                UISceneConfiguration::initWithName_sessionRole(
                    mtm.alloc(),
                    Some(ns_string!("Default Configuration")),
                    &connecting_scene_session.role(),
                )
            }
        }

        #[method(application:didDiscardSceneSessions:)]
        fn _application_did_discard_scene_sessions(
            &self,
            _application: &UIApplication,
            _scene_sessions: &NSSet<UISceneSession>,
        ) {
            tracing::info!("application:didDiscardSceneSessions:");
            // Called when the user discards a scene session.
            // If any sessions were discarded while the application was not running, this will be called shortly after application:didFinishLaunchingWithOptions.
            // Use this method to release any resources that were specific to the discarded scenes, as they will not return.
        }
    }
);
