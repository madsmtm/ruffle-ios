use objc2::ClassType;
use ruffle_ios::{init_logging, launch, AppDelegate};

#[tokio::main]
async fn main() {
    init_logging();
    launch(None, Some(AppDelegate::class()));
}
