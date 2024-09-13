use std::cell::{Cell, OnceCell};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use objc2::rc::{Allocated, Retained};
use objc2::runtime::AnyClass;
use objc2::{declare_class, msg_send_id, mutability, sel, ClassType, DeclaredClass};
use objc2_foundation::{
    CGRect, NSCoder, NSDate, NSObjectProtocol, NSRunLoop, NSRunLoopCommonModes, NSTimer,
};
use objc2_quartz_core::{CALayer, CALayerDelegate, CAMetalLayer};
use objc2_ui_kit::{UIView, UIViewContentMode};
use ruffle_core::{Player, ViewportDimensions};
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::target::SwapChainTarget;

#[derive(Default)]
pub struct Ivars {
    player: OnceCell<Arc<Mutex<Player>>>,
    timer: OnceCell<Retained<NSTimer>>,
    last_frame_time: Cell<Option<Instant>>,
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

        #[method(timerFire:)]
        fn _timer_fire(&self, _timer: &NSTimer) {
            self.timer_fire();
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

        // Create repeating timer that won't fire until we properly start it
        // (because of the high interval).
        let timer = unsafe {
            NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
                f64::MAX,
                self,
                sel!(timerFire:),
                None,
                true,
            )
        };
        // Associate the timer with all run loop modes, so that it runs even
        // when live-resizing or mouse dragging the window.
        unsafe { NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes) };
        self.ivars().timer.set(timer).expect("init timer only once");
    }

    pub fn set_player(&self, player: Arc<Mutex<Player>>) {
        // TODO: Use `player.start_time` here to ensure that our deltas are
        // correct.
        self.ivars().last_frame_time.set(Some(Instant::now()));
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
        tracing::trace!("drawing");
        // Render if the system asks for it because of a resize,
        // or if we asked for it with `setNeedsDisplay`.
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

    #[track_caller]
    pub fn timer(&self) -> &NSTimer {
        self.ivars().timer.get().expect("timer initialized")
    }

    pub fn start(&self) {
        self.player_lock().set_is_playing(true);
        unsafe { self.timer().fire() };
    }

    pub fn stop(&self) {
        self.player_lock().set_is_playing(false);
        // Don't update the timer while we're stopped
        unsafe { self.timer().setFireDate(&NSDate::distantFuture()) };
    }

    fn timer_fire(&self) {
        let last_frame_time = self
            .ivars()
            .last_frame_time
            .get()
            .expect("initialized last frame time");
        let new_time = Instant::now();
        let dt = new_time.duration_since(last_frame_time).as_micros();
        self.ivars().last_frame_time.set(Some(new_time));
        tracing::trace!("timer fire: {:?}", dt as f64 / 1000000.0);

        let mut player_lock = self.player_lock();

        player_lock.tick(dt as f64 / 1000.0);
        // FIXME: The instant that `time_til_next_frame` is relative to isn't
        // defined, so we have to assume that it's roughly relative to "now".
        let next_fire = unsafe {
            NSDate::dateWithTimeIntervalSinceNow(player_lock.time_til_next_frame().as_secs_f64())
        };
        unsafe { self.timer().setFireDate(&next_fire) };

        if player_lock.needs_render() {
            self.layer().setNeedsDisplay();
        }
    }
}

impl Drop for PlayerView {
    fn drop(&mut self) {
        // Invalidate the timer if it was registered
        if let Some(timer) = self.ivars().timer.get() {
            unsafe { timer.invalidate() };
        }
    }
}
