use std::cell::OnceCell;
use std::sync::{Arc, Mutex, MutexGuard};

use objc2::rc::{Allocated, Retained};
use objc2::runtime::AnyClass;
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{CGRect, NSCoder, NSObjectProtocol};
use objc2_quartz_core::{CALayer, CALayerDelegate, CAMetalLayer};
use objc2_ui_kit::{UIView, UIViewContentMode};
use ruffle_core::{Player, ViewportDimensions};
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::target::SwapChainTarget;

#[derive(Default)]
pub struct Ivars {
    player: OnceCell<Arc<Mutex<Player>>>,
}

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

        #[method(layerClass)]
        fn layer_class() -> &AnyClass {
            CAMetalLayer::class()
        }
    }

    // We implement the layer delegate instead of the usual `drawRect:` and
    // `layoutSubviews` methods, since we use a custom `layerClass`, and then
    // UIView won't call those methods.
    //
    // The view is automatically set as the layer's delegate.
    unsafe impl CALayerDelegate for PlayerView {
        #[method(displayLayer:)]
        fn _display_layer(&self, _layer: &CALayer) {
            self.draw_rect();
        }

        // This is the recommended way to listen for changes to the layer's
        // frame. Also tracks changes to the backing scale factor.
        //
        // It may be called at other times though, so we check the configured
        // size in `resize` first to avoid unnecessary work.
        #[method(layoutSublayersOfLayer:)]
        fn _layout_sublayers_of_layer(&self, _layer: &CALayer) {
            self.resize();
        }
    }
);

impl PlayerView {
    #[allow(non_snake_case)]
    pub fn initWithFrame(this: Allocated<Self>, frame_rect: CGRect) -> Retained<Self> {
        unsafe { msg_send_id![this, initWithFrame: frame_rect] }
    }

    fn init(&self) {
        // Ensure that the view calls `drawRect:` after being resized
        unsafe { self.setContentMode(UIViewContentMode::Redraw) };
    }

    pub fn set_player(&self, player: Arc<Mutex<Player>>) {
        self.ivars()
            .player
            .set(player)
            .unwrap_or_else(|_| panic!("only init player once"));
    }

    #[track_caller]
    pub fn player_lock(&self) -> MutexGuard<'_, Player> {
        self.ivars()
            .player
            .get()
            .expect("player initialized")
            .lock()
            .expect("player lock")
    }

    fn resize(&self) {
        tracing::info!("resizing to {:?}", self.frame().size);
        let new_dimensions = self.viewport_dimensions();

        let mut player_lock = self.player_lock();
        // Avoid unnecessary resizes
        let old_dimensions = player_lock.viewport_dimensions();
        if new_dimensions.height != old_dimensions.height
            || new_dimensions.width != old_dimensions.width
            || new_dimensions.scale_factor != old_dimensions.scale_factor
        {
            player_lock.set_viewport_dimensions(new_dimensions);
        }
    }

    fn draw_rect(&self) {
        tracing::info!("drawing");
        self.player_lock().tick(0.5);
        self.player_lock().run_frame();
        self.player_lock().render();
    }

    pub fn viewport_dimensions(&self) -> ViewportDimensions {
        let size = self.frame().size;
        let scale_factor = self.contentScaleFactor();
        ViewportDimensions {
            width: (size.width * scale_factor) as u32,
            height: (size.height * scale_factor) as u32,
            scale_factor: scale_factor as f64,
        }
    }

    pub fn create_renderer(&self) -> WgpuRenderBackend<SwapChainTarget> {
        let layer = self.layer();
        let dimensions = self.viewport_dimensions();
        let layer_ptr = Retained::as_ptr(&layer).cast_mut().cast();
        unsafe {
            WgpuRenderBackend::for_window_unsafe(
                wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr),
                (dimensions.width, dimensions.height),
                wgpu::Backends::METAL,
                wgpu::PowerPreference::HighPerformance,
                None,
            )
            .expect("creating renderer")
        }
    }
}
