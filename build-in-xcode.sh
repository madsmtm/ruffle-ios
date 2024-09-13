# Modified from https://gitlab.com/kornelski/cargo-xcode/-/blob/9b1679b950d16f42eb14fb8446ae1a80e2c867d2/src/xcodebuild.sh

# Nix
if [ -e '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh' ]; then
  source '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh'
fi
# End Nix


set -euo pipefail;
export PATH="$HOME/.cargo/bin:$PATH:/usr/local/bin:/opt/homebrew/bin";
# don't use ios/watchos linker for build scripts and proc macros
export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/ld
export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=/usr/bin/ld
export NO_COLOR=1
OTHER_INPUT_FILE_FLAGS=""

# Make Cargo output cache files in Xcode's directories
export CARGO_TARGET_DIR=$DERIVED_FILE_DIR

case "$PLATFORM_NAME" in
  "macosx")
    CARGO_XCODE_TARGET_OS=darwin
    if [ "${IS_MACCATALYST-NO}" = YES ]; then
      CARGO_XCODE_TARGET_OS=ios-macabi
    fi
    ;;
  "iphoneos") CARGO_XCODE_TARGET_OS=ios ;;
  "iphonesimulator") CARGO_XCODE_TARGET_OS=ios-sim ;;
  "appletvos" | "appletvsimulator") CARGO_XCODE_TARGET_OS=tvos ;;
  "watchos") CARGO_XCODE_TARGET_OS=watchos ;;
  "watchsimulator") CARGO_XCODE_TARGET_OS=watchos-sim ;;
  "xros") CARGO_XCODE_TARGET_OS=visionos ;;
  "xrsimulator") CARGO_XCODE_TARGET_OS=visionos-sim ;;
  *)
    CARGO_XCODE_TARGET_OS="$PLATFORM_NAME"
    echo >&2 "warning: cargo-xcode needs to be updated to handle $PLATFORM_NAME"
    ;;
esac

case "$CONFIGURATION" in
  "Debug")
    CARGO_XCODE_BUILD_PROFILE=debug
  ;;
  "Release")
    CARGO_XCODE_BUILD_PROFILE=release
    OTHER_INPUT_FILE_FLAGS+=" --release"
  ;;
  *)
    echo >&2 "warning: cargo-xcode needs to be updated to handle CONFIGURATION=$CONFIGURATION"
    ;;
esac

CARGO_XCODE_TARGET_TRIPLES=""
CARGO_XCODE_TARGET_FLAGS=""
LIPO_ARGS=""
for arch in $ARCHS; do
  if [[ "$arch" == "arm64" ]]; then arch=aarch64; fi
  if [[ "$arch" == "i386" && "$CARGO_XCODE_TARGET_OS" != "ios" ]]; then arch=i686; fi
  triple="${arch}-apple-$CARGO_XCODE_TARGET_OS"
  CARGO_XCODE_TARGET_TRIPLES+=" $triple"
  CARGO_XCODE_TARGET_FLAGS+=" --target=$triple"
  LIPO_ARGS+="$CARGO_TARGET_DIR/$triple/$CARGO_XCODE_BUILD_PROFILE/$EXECUTABLE_NAME
"
done

echo >&2 "Cargo $CONFIGURATION $ACTION for $PLATFORM_NAME $ARCHS =$CARGO_XCODE_TARGET_TRIPLES; using ${SDK_NAMES:-}. \$PATH is:"
tr >&2 : '\n' <<<"$PATH"

if [ "$ACTION" = clean ]; then
  cargo clean --verbose $CARGO_XCODE_TARGET_FLAGS $OTHER_INPUT_FILE_FLAGS;
  rm -f "$SCRIPT_OUTPUT_FILE_0"
  exit 0
fi

{ cargo build --features="${CARGO_XCODE_FEATURES:-}" $CARGO_XCODE_TARGET_FLAGS $OTHER_INPUT_FILE_FLAGS --verbose --message-format=short 2>&1 | sed -E 's/^([^ :]+:[0-9]+:[0-9]+: error)/\1: /' >&2; } || { echo >&2 "error: cargo-xcode project build failed; $CARGO_XCODE_TARGET_TRIPLES"; exit 1; }

tr '\n' '\0' <<<"$LIPO_ARGS" | xargs -0 lipo -create -output "$SCRIPT_OUTPUT_FILE_0"

if [ ${LD_DYLIB_INSTALL_NAME:+1} ]; then
  install_name_tool -id "$LD_DYLIB_INSTALL_NAME" "$SCRIPT_OUTPUT_FILE_0"
fi

DEP_FILE_DST="$DERIVED_FILE_DIR/${ARCHS}-${EXECUTABLE_NAME}.d"
echo "" > "$DEP_FILE_DST"
for triple in $CARGO_XCODE_TARGET_TRIPLES; do
  BUILT_SRC="$CARGO_TARGET_DIR/$triple/$CARGO_XCODE_BUILD_PROFILE/$EXECUTABLE_NAME"

  # cargo generates a dep file, but for its own path, so append our rename to it
  DEP_FILE_SRC="$CARGO_TARGET_DIR/$triple/$CARGO_XCODE_BUILD_PROFILE/$EXECUTABLE_NAME.d"
  if [ -f "$DEP_FILE_SRC" ]; then
    cat "$DEP_FILE_SRC" >> "$DEP_FILE_DST"
  fi
  echo >> "$DEP_FILE_DST" "${SCRIPT_OUTPUT_FILE_0/ /\\ /}: ${BUILT_SRC/ /\\ /}"
done
cat "$DEP_FILE_DST"

echo "success: $ACTION of $SCRIPT_OUTPUT_FILE_0 for $CARGO_XCODE_TARGET_TRIPLES"
