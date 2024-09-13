use std::cell::Cell;

use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::NSObjectProtocol;
use objc2_ui_kit::{
    UIResponder, UIScene, UISceneConnectionOptions, UISceneDelegate, UISceneSession, UIWindow,
    UIWindowSceneDelegate,
};

pub struct Ivars {
    window: Cell<Option<Retained<UIWindow>>>,
}

declare_class!(
    pub struct SceneDelegate;

    unsafe impl ClassType for SceneDelegate {
        type Super = UIResponder;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "SceneDelegate";
    }

    impl DeclaredClass for SceneDelegate {
        type Ivars = Ivars;
    }

    unsafe impl SceneDelegate {
        // Called by UIStoryboard
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            tracing::info!("init scene");
            let this = this.set_ivars(Ivars { window: Cell::new(None), });
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for SceneDelegate {}

    #[allow(non_snake_case)]
    unsafe impl UISceneDelegate for SceneDelegate {
        #[method(scene:willConnectToSession:options:)]
        fn scene_willConnectToSession_options(
            &self,
            _scene: &UIScene,
            _session: &UISceneSession,
            _connection_options: &UISceneConnectionOptions,
        ) {
            tracing::info!("scene:willConnectToSession:options:");
            // Use this method to optionally configure and attach the UIWindow `window` to the provided UIWindowScene `scene`.
            // If using a storyboard, the `window` property will automatically be initialized and attached to the scene.
            // This delegate does not imply the connecting scene or session are new (see `application:configurationForConnectingSceneSession` instead).
        }

        #[method(sceneDidDisconnect:)]
        fn sceneDidDisconnect(&self, _scene: &UIScene) {
            tracing::info!("sceneDidDisconnect:");
            // Called as the scene is being released by the system.
            // This occurs shortly after the scene enters the background, or when its session is discarded.
            // Release any resources associated with this scene that can be re-created the next time the scene connects.
            // The scene may re-connect later, as its session was not necessarily discarded (see `application:didDiscardSceneSessions` instead).
        }

        #[method(sceneDidBecomeActive:)]
        fn sceneDidBecomeActive(&self, _scene: &UIScene) {
            tracing::info!("sceneDidBecomeActive:");
            // Called when the scene has moved from an inactive state to an active state.
            // Use this method to restart any tasks that were paused (or not yet started) when the scene was inactive.
        }

        #[method(sceneWillResignActive:)]
        fn sceneWillResignActive(&self, _scene: &UIScene) {
            tracing::info!("sceneWillResignActive:");
            // Called when the scene will move from an active state to an inactive state.
            // This may occur due to temporary interruptions (ex. an incoming phone call).
        }

        #[method(sceneWillEnterForeground:)]
        fn sceneWillEnterForeground(&self, _scene: &UIScene) {
            tracing::info!("sceneWillEnterForegrounds:");
            // Called as the scene transitions from the background to the foreground.
            // Use this method to undo the changes made on entering the background.
        }

        #[method(sceneDidEnterBackground:)]
        fn sceneDidEnterBackground(&self, _scene: &UIScene) {
            tracing::info!("sceneDidEnterBackground:");
            // Called as the scene transitions from the foreground to the background.
            // Use this method to save data, release shared resources, and store enough scene-specific state information
            // to restore the scene back to its current state.
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UIWindowSceneDelegate for SceneDelegate {
        #[method_id(window)]
        fn window(&self) -> Option<Retained<UIWindow>> {
            let window = self.ivars().window.take();
            self.ivars().window.set(window.clone());
            window
        }

        #[method(setWindow:)]
        fn setWindow(&self, window: Option<&UIWindow>) {
            self.ivars().window.set(window.map(|w| w.retain()));
        }
    }
);

impl Drop for SceneDelegate {
    fn drop(&mut self) {
        tracing::info!("drop scene");
    }
}
