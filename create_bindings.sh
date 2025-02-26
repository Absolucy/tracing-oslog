#!/usr/bin/env bash

set -e

clang_targets_and_sdk=(
    "x86_64-apple-macosx10.12 macosx"
    "arm64-apple-macosx10.12 macosx"
    "i386-apple-macosx10.12 macosx"
    "arm64-apple-ios13.1-macabi macosx"

    "arm64-apple-ios10.0 iphoneos"
    "x86_64-apple-ios10.0-simulator iphonesimulator"
    "armv7s-apple-ios10.0 iphoneos"

    "arm64-apple-tvos10.0 appletvos"
    "arm64-apple-watchos3.0 watchos"
    "arm64-apple-visionos1.0 xros"
)

mkdir -p target/bindings

for i in "${!clang_targets_and_sdk[@]}";
do
    set -- ${clang_targets_and_sdk[$i]}
    target=$1
    sdk=$2
    echo "$target (SDK $sdk)"

    # Generate bindings for each supported target.
    #
    # `--no-layout-tests`: Layout tests will fail on 32-bit.
    # `-DOS_ACTIVITY_OBJECT_API=1`: Ensure that we generate APIs like `os_activity_scope_enter`.
    xcrun --sdk $sdk bindgen wrapper.h \
        --allowlist-function "_?os_activity_.*" \
        --allowlist-function "os_log_.*" \
        --allowlist-function "os_release" \
        --allowlist-function "wrapped_.*" \
        --allowlist-type "os_activity_.*" \
        --allowlist-type "os_log_.*" \
        --allowlist-var "_?os_activity_.*" \
        --allowlist-var "__dso_handle" \
        --no-layout-tests \
        --rust-target 1.77 \
        -- -target $target -DOS_ACTIVITY_OBJECT_API=1 > "target/bindings/$target.rs"

    # Copy first to destination.
    if [ "$i" -eq 0 ]; then
        cp "target/bindings/$target.rs" src/bindings.rs
    fi

    # Ensure that the generated bindings are the same for all other targets.
    diff "target/bindings/$target.rs" src/bindings.rs
done
