# The Ruffle Flash Player emulator on iOS


## Design choices

A normal person might have wrapped the Rust in some `extern "C" fn`s, and then used SwiftUI, or at least Objective-C for the UI shell. I would probably recommend that for most use-cases.

I'm developing [`objc2`](https://github.com/madsmtm/objc2) though, and I want to improve the user-interface of that, so I decided to be a bit unortodox, and do everything in Rust.

## Testing

Run on Mac Catalyst with:
```
cargo +nightly bundle --target=aarch64-apple-ios-macabi && ./target/aarch64-apple-ios-macabi/debug/bundle/ios/Ruffle.app/ruffle-ios
```
