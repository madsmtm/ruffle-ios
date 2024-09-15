use std::cell::{Cell, OnceCell};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use objc2::rc::{Allocated, Retained};
use objc2::runtime::AnyClass;
use objc2::{declare_class, msg_send, msg_send_id, mutability, sel, ClassType, DeclaredClass};
use objc2_foundation::{
    CGRect, MainThreadMarker, NSCoder, NSDate, NSObjectProtocol, NSRunLoop, NSRunLoopCommonModes,
    NSSet, NSTimer,
};
use objc2_quartz_core::{CALayer, CALayerDelegate, CAMetalLayer};
use objc2_ui_kit::{
    UIEvent, UIKeyboardHIDUsage, UIPress, UIPressPhase, UIPressesEvent, UITouch, UITouchPhase,
    UIView, UIViewContentMode,
};
use ruffle_core::events::{KeyCode, MouseButton};
use ruffle_core::{Player, PlayerEvent, ViewportDimensions};
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::target::SwapChainTarget;

#[derive(Default)]
pub struct Ivars {
    player: OnceCell<Arc<Mutex<Player>>>,
    timer: OnceCell<Retained<NSTimer>>,
    last_frame_time: Cell<Option<Instant>>,
}

declare_class!(
    #[derive(Debug)]
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

    // UIResponder
    #[allow(non_snake_case)]
    unsafe impl PlayerView {
        #[method(canBecomeFirstResponder)]
        fn canBecomeFirstResponder(&self) -> bool {
            true
        }

        #[method(becomeFirstResponder)]
        fn becomeFirstResponder(&self) -> bool {
            tracing::info!("becomeFirstResponder");
            true
        }

        #[method(canResignFirstResponder)]
        fn canResignFirstResponder(&self) -> bool {
            true
        }

        #[method(resignFirstResponder)]
        fn resignFirstResponder(&self) -> bool {
            tracing::info!("resignFirstResponder");
            true
        }

        #[method(touchesBegan:withEvent:)]
        fn touchesBegan_withEvent(
            &self,
            touches: &NSSet<UITouch>,
            event: Option<&UIEvent>,
        ) {
            tracing::trace!("touchesBegan:withEvent:");
            if !self.handle_touches(touches) {
                // Forward to super
                let _: () = unsafe { msg_send![super(self), touchesBegan: touches, withEvent: event] };
            }
        }

        #[method(touchesMoved:withEvent:)]
        fn touchesMoved_withEvent(
            &self,
            touches: &NSSet<UITouch>,
            event: Option<&UIEvent>,
        ) {
            tracing::trace!("touchesMoved:withEvent:");
            if !self.handle_touches(touches) {
                // Forward to super
                let _: () = unsafe { msg_send![super(self), touchesMoved: touches, withEvent: event] };
            }
        }

        #[method(touchesEnded:withEvent:)]
        fn touchesEnded_withEvent(
            &self,
            touches: &NSSet<UITouch>,
            event: Option<&UIEvent>,
        ) {
            tracing::trace!("touchesEnded:withEvent:");
            if !self.handle_touches(touches) {
                // Forward to super
                let _: () = unsafe { msg_send![super(self), touchesEnded: touches, withEvent: event] };
            }
        }

        #[method(touchesCancelled:withEvent:)]
        fn touchesCancelled_withEvent(
            &self,
            touches: &NSSet<UITouch>,
            event: Option<&UIEvent>,
        ) {
            tracing::trace!("touchesCancelled:withEvent:");
            if !self.handle_touches(touches) {
                // Forward to super
                let _: () = unsafe { msg_send![super(self), touchesCancelled: touches, withEvent: event] };
            }
        }

        #[method(pressesBegan:withEvent:)]
        fn pressesBegan_withEvent(
            &self,
            presses: &NSSet<UIPress>,
            event: Option<&UIPressesEvent>,
        ) {
            tracing::trace!("pressesBegan:withEvent:");
            if !self.handle_presses(presses) {
                // Forward to super
                let _: () = unsafe { msg_send![super(self), pressesBegan: presses, withEvent: event] };
            }
        }

        #[method(pressesChanged:withEvent:)]
        fn pressesChanged_withEvent(
            &self,
            presses: &NSSet<UIPress>,
            event: Option<&UIPressesEvent>,
        ) {
            tracing::trace!("pressesChanged:withEvent:");
            if !self.handle_presses(presses) {
                // Forward to super
                let _: () = unsafe { msg_send![super(self), pressesChanged: presses, withEvent: event] };
            }
        }

        #[method(pressesEnded:withEvent:)]
        fn pressesEnded_withEvent(
            &self,
            presses: &NSSet<UIPress>,
            event: Option<&UIPressesEvent>,
        ) {
            tracing::trace!("pressesEnded:withEvent:");
            if !self.handle_presses(presses) {
                // Forward to super
                let _: () = unsafe { msg_send![super(self), pressesEnded: presses, withEvent: event] };
            }
        }

        #[method(pressesCancelled:withEvent:)]
        fn pressesCancelled_withEvent(
            &self,
            presses: &NSSet<UIPress>,
            event: Option<&UIPressesEvent>,
        ) {
            tracing::trace!("pressesCancelled:withEvent:");
            if !self.handle_presses(presses) {
                // Forward to super
                let _: () = unsafe { msg_send![super(self), pressesCancelled: presses, withEvent: event] };
            }
        }

        #[method(remoteControlReceivedWithEvent:)]
        fn remoteControlReceivedWithEvent(&self, event: Option<&UIEvent>) {
            tracing::info!(subtype = ?event.map(|e| unsafe { e.subtype() }), "remoteControlReceivedWithEvent:");
        }
    }

    // UIView
    #[allow(non_snake_case)]
    unsafe impl PlayerView {
        #[method(canBecomeFocused)]
        fn canBecomeFocused(&self) -> bool {
            tracing::info!("canBecomeFocused");
            true
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
        //
        // TODO: Consider running two timers, one to maintain the frame rate,
        // and one to update Flash timers.
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
        // FIXME: Expose `PartialEq` on `ViewportDimensions`.
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
                (dimensions.width.max(1), dimensions.height.max(1)),
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

    fn handle_touches(&self, touches: &NSSet<UITouch>) -> bool {
        let mut player_lock = self.player_lock();

        // Flash only supports one touch at a time, so we intentially don't set
        // `multipleTouchEnabled`, and don't have to do check all touches here.
        let touch = unsafe { touches.anyObject().expect("touches must contain a touch") };

        let point = touch.locationInView(Some(self));
        let scale_factor = self.contentScaleFactor();
        let x = point.x as f64 * scale_factor;
        let y = point.y as f64 * scale_factor;
        // We don't know which button was pressed in UIKit.
        let button = MouseButton::Left;

        let event_handled = match touch.phase() {
            UITouchPhase::Began => {
                player_lock.set_mouse_in_stage(true);
                player_lock.handle_event(PlayerEvent::MouseDown {
                    x,
                    y,
                    button,
                    // We always know whether a click was a double click or not.
                    index: Some(touch.tapCount()),
                })
            }
            UITouchPhase::Moved => {
                player_lock.set_mouse_in_stage(true);
                player_lock.handle_event(PlayerEvent::MouseMove { x, y })
            }
            UITouchPhase::Ended => {
                player_lock.set_mouse_in_stage(true);
                let up_handled = player_lock.handle_event(PlayerEvent::MouseUp { x, y, button });
                player_lock.set_mouse_in_stage(false);
                up_handled || player_lock.handle_event(PlayerEvent::MouseLeave)
            }
            UITouchPhase::Cancelled => {
                player_lock.set_mouse_in_stage(true);
                player_lock.handle_event(PlayerEvent::MouseLeave)
            }
            _ => return false,
        };

        if player_lock.needs_render() {
            self.layer().setNeedsDisplay();
        }

        event_handled
    }

    fn handle_presses(&self, presses: &NSSet<UIPress>) -> bool {
        let mtm = MainThreadMarker::from(self);
        let mut player_lock = self.player_lock();

        let mut handled = false;
        for press in presses {
            // TODO: Consider press.r#type()
            let Some(key) = (unsafe { press.key(mtm) }) else {
                continue;
            };
            let key_code = key_code_to_ruffle(unsafe { key.keyCode() });
            // FIXME: `last()` is functionally equivalent in most cases, but
            // we may want to do something else here.
            let key_char = unsafe { key.charactersIgnoringModifiers() }
                .to_string()
                .chars()
                .last();

            let event = match unsafe { press.phase() } {
                UIPressPhase::Began => PlayerEvent::KeyDown { key_code, key_char },
                // FIXME: Forward event cancellation
                UIPressPhase::Ended | UIPressPhase::Cancelled => {
                    PlayerEvent::KeyUp { key_code, key_char }
                }
                _ => continue,
            };

            handled |= player_lock.handle_event(event);
        }
        handled
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

fn key_code_to_ruffle(code: UIKeyboardHIDUsage) -> KeyCode {
    use objc2_ui_kit::UIKeyboardHIDUsage as UI;
    match code {
        UI::KeyboardA => KeyCode::A,
        UI::KeyboardB => KeyCode::B,
        UI::KeyboardC => KeyCode::C,
        UI::KeyboardD => KeyCode::D,
        UI::KeyboardE => KeyCode::E,
        UI::KeyboardF => KeyCode::F,
        UI::KeyboardG => KeyCode::G,
        UI::KeyboardH => KeyCode::H,
        UI::KeyboardI => KeyCode::I,
        UI::KeyboardJ => KeyCode::J,
        UI::KeyboardK => KeyCode::K,
        UI::KeyboardL => KeyCode::L,
        UI::KeyboardM => KeyCode::M,
        UI::KeyboardN => KeyCode::N,
        UI::KeyboardO => KeyCode::O,
        UI::KeyboardP => KeyCode::P,
        UI::KeyboardQ => KeyCode::Q,
        UI::KeyboardR => KeyCode::R,
        UI::KeyboardS => KeyCode::S,
        UI::KeyboardT => KeyCode::T,
        UI::KeyboardU => KeyCode::U,
        UI::KeyboardV => KeyCode::V,
        UI::KeyboardW => KeyCode::W,
        UI::KeyboardX => KeyCode::X,
        UI::KeyboardY => KeyCode::Y,
        UI::KeyboardZ => KeyCode::Z,
        UI::Keyboard1 => KeyCode::KEY1,
        UI::Keyboard2 => KeyCode::KEY2,
        UI::Keyboard3 => KeyCode::KEY3,
        UI::Keyboard4 => KeyCode::KEY4,
        UI::Keyboard5 => KeyCode::KEY5,
        UI::Keyboard6 => KeyCode::KEY6,
        UI::Keyboard7 => KeyCode::KEY7,
        UI::Keyboard8 => KeyCode::KEY8,
        UI::Keyboard9 => KeyCode::KEY9,
        UI::Keyboard0 => KeyCode::KEY0,
        UI::KeyboardReturnOrEnter => KeyCode::RETURN,
        UI::KeyboardEscape => KeyCode::ESCAPE,
        UI::KeyboardDeleteOrBackspace => KeyCode::DELETE,
        UI::KeyboardTab => KeyCode::TAB,
        UI::KeyboardSpacebar => KeyCode::SPACE,
        UI::KeyboardHyphen => KeyCode::MINUS,
        UI::KeyboardEqualSign => KeyCode::EQUALS,
        UI::KeyboardOpenBracket => KeyCode::LBRACKET,
        UI::KeyboardCloseBracket => KeyCode::RBRACKET,
        UI::KeyboardBackslash => KeyCode::BACKSLASH,
        UI::KeyboardSemicolon => KeyCode::SEMICOLON,
        UI::KeyboardQuote => KeyCode::APOSTROPHE,
        UI::KeyboardGraveAccentAndTilde => KeyCode::GRAVE,
        UI::KeyboardComma => KeyCode::COMMA,
        UI::KeyboardPeriod => KeyCode::PERIOD,
        UI::KeyboardSlash => KeyCode::SLASH,
        UI::KeyboardCapsLock => KeyCode::CAPS_LOCK,
        UI::KeyboardF1 => KeyCode::F1,
        UI::KeyboardF2 => KeyCode::F2,
        UI::KeyboardF3 => KeyCode::F3,
        UI::KeyboardF4 => KeyCode::F4,
        UI::KeyboardF5 => KeyCode::F5,
        UI::KeyboardF6 => KeyCode::F6,
        UI::KeyboardF7 => KeyCode::F7,
        UI::KeyboardF8 => KeyCode::F8,
        UI::KeyboardF9 => KeyCode::F9,
        UI::KeyboardF10 => KeyCode::F10,
        UI::KeyboardF11 => KeyCode::F11,
        UI::KeyboardF12 => KeyCode::F12,
        UI::KeyboardScrollLock => KeyCode::SCROLL_LOCK,
        UI::KeyboardPause => KeyCode::PAUSE,
        UI::KeyboardInsert => KeyCode::INSERT,
        UI::KeyboardHome => KeyCode::HOME,
        UI::KeyboardPageUp => KeyCode::PG_UP,
        UI::KeyboardEnd => KeyCode::END,
        UI::KeyboardPageDown => KeyCode::PG_DOWN,
        UI::KeyboardRightArrow => KeyCode::RIGHT,
        UI::KeyboardLeftArrow => KeyCode::LEFT,
        UI::KeyboardDownArrow => KeyCode::DOWN,
        UI::KeyboardUpArrow => KeyCode::UP,
        UI::KeypadNumLock => KeyCode::NUM_LOCK,
        UI::KeypadSlash => KeyCode::NUMPAD_SLASH,
        UI::KeypadAsterisk => KeyCode::MULTIPLY,
        UI::KeypadHyphen => KeyCode::NUMPAD_MINUS,
        UI::KeypadPlus => KeyCode::PLUS,
        UI::KeypadEnter => KeyCode::NUMPAD_ENTER,
        UI::Keypad1 => KeyCode::NUMPAD1,
        UI::Keypad2 => KeyCode::NUMPAD2,
        UI::Keypad3 => KeyCode::NUMPAD3,
        UI::Keypad4 => KeyCode::NUMPAD4,
        UI::Keypad5 => KeyCode::NUMPAD5,
        UI::Keypad6 => KeyCode::NUMPAD6,
        UI::Keypad7 => KeyCode::NUMPAD7,
        UI::Keypad8 => KeyCode::NUMPAD8,
        UI::Keypad9 => KeyCode::NUMPAD9,
        UI::Keypad0 => KeyCode::NUMPAD0,
        UI::KeypadPeriod => KeyCode::NUMPAD_PERIOD,
        UI::KeyboardNonUSBackslash => KeyCode::BACKSLASH,
        UI::KeypadEqualSign => KeyCode::EQUALS,
        UI::KeyboardF13 => KeyCode::F13,
        UI::KeyboardF14 => KeyCode::F14,
        UI::KeyboardF15 => KeyCode::F15,
        UI::KeyboardF16 => KeyCode::F16,
        UI::KeyboardF17 => KeyCode::F17,
        UI::KeyboardF18 => KeyCode::F18,
        UI::KeyboardF19 => KeyCode::F19,
        UI::KeyboardF20 => KeyCode::F20,
        UI::KeyboardF21 => KeyCode::F21,
        UI::KeyboardF22 => KeyCode::F22,
        UI::KeyboardF23 => KeyCode::F23,
        UI::KeyboardF24 => KeyCode::F24,
        UI::KeypadComma => KeyCode::COMMA,
        UI::KeypadEqualSignAS400 => KeyCode::EQUALS,
        UI::KeyboardInternational1 => KeyCode::KEY1,
        UI::KeyboardInternational2 => KeyCode::KEY2,
        UI::KeyboardInternational3 => KeyCode::KEY3,
        UI::KeyboardInternational4 => KeyCode::KEY4,
        UI::KeyboardInternational5 => KeyCode::KEY5,
        UI::KeyboardInternational6 => KeyCode::KEY6,
        UI::KeyboardInternational7 => KeyCode::KEY7,
        UI::KeyboardInternational8 => KeyCode::KEY8,
        UI::KeyboardInternational9 => KeyCode::KEY9,
        UI::KeyboardReturn => KeyCode::RETURN,
        UI::KeyboardLeftControl => KeyCode::CONTROL,
        UI::KeyboardLeftShift => KeyCode::SHIFT,
        UI::KeyboardLeftAlt => KeyCode::ALT,
        UI::KeyboardLeftGUI => KeyCode::COMMAND,
        UI::KeyboardRightControl => KeyCode::CONTROL,
        UI::KeyboardRightShift => KeyCode::SHIFT,
        UI::KeyboardRightAlt => KeyCode::ALT,
        UI::KeyboardRightGUI => KeyCode::COMMAND,
        _ => {
            tracing::warn!("unhandled key {}", code.0);
            KeyCode::UNKNOWN
        }
    }
}
