#/usr/bin/env bash

APP_NAME="$(cat Cargo.toml | dasel -r toml '.package.name')"
BINARY_NAME="$(cat Cargo.toml | dasel -r toml '.package.default-run')"
BUNDLE_ID="$(cat Cargo.toml | dasel -r toml '.package.metadata.bundle.bin.app.identifier')"

cargo bundle --bin $BINARY_NAME --target aarch64-apple-ios-sim
mv "target/aarch64-apple-ios-sim/debug/bundle/ios/$APP_NAME.app/$APP_NAME" "target/aarch64-apple-ios-sim/debug/bundle/ios/$APP_NAME.app/$BINARY_NAME"
xcrun simctl install booted "target/aarch64-apple-ios-sim/debug/bundle/ios/$APP_NAME.app"
xcrun simctl launch --console booted "$BUNDLE_ID"