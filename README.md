# The Ruffle Flash Player emulator on iOS


## Design choices

A normal person might have wrapped the Rust in some `extern "C" fn`s, and then used SwiftUI, or at least Objective-C for the UI shell. I would probably recommend that for most use-cases.

I'm developing [`objc2`](https://github.com/madsmtm/objc2) though, and I want to improve the user-interface of that, so I decided to be a bit unortodox, and do everything in Rust.

## Testing

Run the core player on Mac Catalyst with:
```
cargo bundle --target=aarch64-apple-ios-macabi --bin run_swf && ./target/aarch64-apple-ios-macabi/debug/bundle/ios/Ruffle.app/run_swf
```

## UI

Similar to https://getutm.app/, we should have:
- A library of "installed" SWFs/bundles/saved links, editable.
- When selecting an SWF, the navigation bar at the top shows various options
  - Opening keyboard (maybe?)
  - Context menu "play, rewind, forward, back, etc."?
  - Allow changing between scale
  - Back button to go back to library

## Library item settings

Settings are stored per SWF / per "Ruffle Bundle" / per saved link. The UI does not significantly differentiate between these.

- Inputs:
  - Configurable
  - Swipe for arrow keys?
  - https://openemu.org/ does it pretty well, equivalent for iOS?
- Player settings:
  - https://github.com/ruffle-rs/ruffle/blob/master/frontend-utils/src/bundle/README.md#player


## Terminology

What do we call an swf? "Game"? "Movie"? "SWF"? "Flash Animation"?


## Plan

1. Get the Ruffle UI running in a `UIView`
2. Wire up some way to start it using an SWF on the local device


## TODO

- Set `idleTimerDisabled` at the appropriate time


## Choices

- Intentionally use `public.app-category.games` to get better performance ("Game Mode" on macOS).
  - This is not necessarily the correct choice for Ruffle, but it's the closest.
- No tab bar, not really desired, since we generally want the SWF's UI to fill most of the screen
- A navigation bar is useful though
  - Also display settings for the current swf
  - Hide when entering full screen
