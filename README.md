# The Ruffle Flash Player emulator on iOS

Work in progress.

See [ruffle.rs](https://ruffle.rs/) for a general introduction.


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
- "Add" and "edit" are two different flows, and should show two different UIs
  - "Add" doesn't have to show all the extra settings; it is only about getting the file. The user can edit it later.

## Library item settings

Settings are stored per Ruffle Bundle.

- `PlayerOptions`
  - https://github.com/ruffle-rs/ruffle/blob/master/frontend-utils/src/bundle/README.md#player
- Inputs:
  - Configurable
  - Swipe for arrow keys?
  - https://openemu.org/ does it pretty well, equivalent for iOS?
- Custom name?
- Custom image?


## Storage ideas

Goals:
- Store a Ruffle bundle.
- Store user settings.
- Store data the SWF itself may have stored (the key-value store).
- Sync to iCloud.
- Be backwards and forwards compatible with new versions of the Ruffle app.
  - Upheld for [Ruffle Bundles](https://discord.com/channels/610531541889581066/1225519553916829736/1232031955751665777).

So, we want to support several modes of launching:
- Import bundle permanently.
- Open a Ruffle Bundle without importing.
- What happens when a bundle has moved relative to the user settings?

Bundles, when imported, are unpackaged from zip, renamed to `bundle.ruf` and moved to `library/$random_uuid/`, to not conflict with other bundles with the same name. Imported SWFs are converted to a Ruffle bundle with `name = "file_stem", url = "file:///file_stem.swf"`. The files are stored on disk in the application's directory. User settings are stored in `settings.toml` next to `bundle.ruf`. Application data in `app_data/`.

Rule for syncing is "newest wins". This _should_ be fine if e.g. the user has modified their settings on two different devices, though might require different logic for application data.

Note that we _could_ have used a Core Data model, but that's difficult and won't really help us when our settings is mostly defined by the Ruffle Bundle.


## Terminology

What do we call an swf? "Game"? "Movie"? "SWF"? "Flash Animation"?


## Plan

1. Get the Ruffle UI running in a `UIView`
2. Wire up some way to start it using an SWF on the local device


## TODO

- Set `idleTimerDisabled` at the appropriate time
- Use white for labels, orange for buttons
- Add settings button in library item

## Choices

- Intentionally use `public.app-category.games` to get better performance ("Game Mode" on macOS).
  - This is not necessarily the correct choice for Ruffle, but it's the closest.
- It doesn't make sense to have root settings like in the desktop version
- No tab bar, not really desired, since we generally want the SWF's UI to fill most of the screen
  - Though if we decide to add an easy way to download from "trusted" sources, we could add a tab bar for that
- A navigation bar is useful though
  - To display some settings for the current swf
  - To go back to library
  - Hide when entering full screen?
