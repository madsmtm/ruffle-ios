# The Ruffle Flash Player emulator on iOS


## Design choices

A normal person might have wrapped the Rust in some `extern "C" fn`s, and then used SwiftUI, or at least Objective-C for the UI shell. I would probably recommend that for most use-cases.

I'm developing [`objc2`](https://github.com/madsmtm/objc2) though, and I want to improve the user-interface of that, so I decided to be a bit unortodox, and do everything in Rust.

## Testing

Run on Mac Catalyst with:
```
cargo +nightly bundle --target=aarch64-apple-ios-macabi && ./target/aarch64-apple-ios-macabi/debug/bundle/ios/Ruffle.app/ruffle-ios
```

## GUI

- Button for opening keyboard, maybe?
- Make sure to respect safe area
- Where do we put the "play, rewind, forward, back, etc." menu?
- How do we do scaling? Should the user be able to zoom?
- Inputs:
  - Configurable?
  - Swipe for arrow keys?
  - https://openemu.org/ does it pretty well, equivalent for iOS?
- UI: https://getutm.app/ has an interface at the top
  - That's hard to reach tho
- Game library?

Plan:
1. Get the Ruffle UI running in a `UIView`
2. Wire up some way to start it using an SWF on the local device


## Choices

- Intentionally use `public.app-category.games` to get better performance ("Game Mode" on macOS).
  - This is not necessarily the correct choice for Ruffle, but it's the closest.
