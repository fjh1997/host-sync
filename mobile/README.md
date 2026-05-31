# HostSync Mobile

## Android

The Android app is a native Kotlin + Jetpack Compose project that calls into the Rust core library via JNI.

### Termux Integration

HostSync uses Termux for SSH connections on Android. The app sends commands to Termux via the `RUN_COMMAND` intent.

#### Prerequisites

1. Install [Termux](https://f-droid.org/packages/com.termux/) from F-Droid
2. Grant `com.termux.permission.RUN_COMMAND` permission:
   ```bash
   adb shell su -c "pm grant com.hostsync.app com.termux.permission.RUN_COMMAND"
   ```
3. In Termux, enable external apps:
   ```bash
   echo "allow-external-apps = true" >> ~/.termux/termux.properties
   ```
4. Restart Termux

#### SSH Key Paths

On Android, SSH keys are stored in Termux's directory:
- Termux home: `/data/data/com.termux/files/home/`
- SSH keys: `/data/data/com.termux/files/home/.ssh/`

If syncing keys from desktop, the path `~/.ssh/id_rsa` on desktop maps to `/data/data/com.termux/files/home/.ssh/id_rsa` in Termux.

For cross-device sync, use the "Private Key" field to store the key content inline (encrypted and synced).

### Build Steps

1. Build the Rust shared library for Android targets:
   ```bash
   # Install Android NDK targets
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

   # Build (requires cargo-ndk)
   cargo install cargo-ndk
   cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o mobile/android/app/src/main/jniLibs build --release -p hostsync-core
   ```

2. Open `mobile/android/` in Android Studio and build normally.

## iOS

The iOS app is a native Swift + SwiftUI project that calls into the Rust core library via C FFI.

### Build Steps

1. Build the Rust static library for iOS targets:
   ```bash
   rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

   cargo build --release --target aarch64-apple-ios -p hostsync-core
   cargo build --release --target aarch64-apple-ios-sim -p hostsync-core

   # Create universal library
   lipo -create \
     target/aarch64-apple-ios/release/libhostsync_core.a \
     target/aarch64-apple-ios-sim/release/libhostsync_core.a \
     -output mobile/ios/libhostsync_core.a
   ```

2. Generate C header:
   ```bash
   cbindgen --crate hostsync-core --output mobile/ios/hostsync_core.h --lang c
   ```

3. Open `mobile/ios/HostSync.xcodeproj` in Xcode and build.
