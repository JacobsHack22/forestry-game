## Build for IOS

Requirements:

- dasel
- cargo-bundle
- xcode 
- ios-deploy

Install dependencies:

```bash
brew install dasel
cargo install cargo-bundle

# for production
rustup target add aarch64-apple-ios

# for development
rustup target add aarch64-apple-ios-sim
```

Set environment variables:

```bash
APP_NAME="$(cat Cargo.toml | dasel -r toml '.package.name')"
BINARY_NAME="$(cat Cargo.toml | dasel -r toml '.package.default-run')"
BUNDLE_ID="$(cat Cargo.toml | dasel -r toml '.package.metadata.bundle.bin.app.identifier')"
```

Boot simulator

```bash
xcrun simctl boot "iPhone 14"
open /Applications/Xcode.app/Contents/Developer/Applications/Simulator.app
```

Build and run on simulator:

```bash
cargo bundle --bin $BINARY_NAME --target aarch64-apple-ios-sim
mv "target/aarch64-apple-ios-sim/debug/bundle/ios/$APP_NAME.app/$APP_NAME" "target/aarch64-apple-ios-sim/debug/bundle/ios/$APP_NAME.app/$BINARY_NAME"
xcrun simctl install booted "target/aarch64-apple-ios-sim/debug/bundle/ios/$APP_NAME.app"
xcrun simctl launch --console booted "$BUNDLE_ID"
```

Build and run on device:

- First compile rust for needed architecture:
  ```bash
  cargo bundle --bin $BINARY_NAME --target aarch64-apple-ios-sim
  mv "target/aarch64-apple-ios/debug/bundle/ios/$APP_NAME.app/$APP_NAME" "target/aarch64-apple-ios/debug/bundle/ios/$APP_NAME.app/$BINARY_NAME"
  ```
  Get an `.app` file as a result.
- Then sign and add provisioning profile to the app using [IOS App Signer](https://github.com/DanTheMan827/ios-app-signer).
- Finally, install the app on the device
  ```bash
  ios-deploy --debug --bundle forestry-rust.ipa
  ```
