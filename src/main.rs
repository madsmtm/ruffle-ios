use objc2::ClassType;
use ruffle_ios::{init_logging, launch, AppDelegate, LogoView, PlayerController, PlayerView};

#[tokio::main]
async fn main() {
    init_logging();

    // Initialize classes defined in Rust
    let _ = PlayerView::class();
    let _ = LogoView::class();
    let _ = PlayerController::class();

    launch(None, Some(AppDelegate::class()));
}
