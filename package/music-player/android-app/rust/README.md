# musicplayer-native

The native audio engine of the Android music player, built as `libmusicplayer_native.so` and loaded by the Kotlin
app with `System.loadLibrary("musicplayer_native")`.
The crate is private (`publish = false`) and standalone: the empty `[workspace]` table in `Cargo.toml` keeps it its own
workspace root, because the Android package is a Gradle build that no parent cargo workspace owns.

## What it provides

- Decoding of FLAC, WAV, MP3, Vorbis, AAC and ALAC through `symphonia`, and Opus through `libopus` (`src/decode.rs`,
  `src/opus.rs`), from a path or an Android `content://` file descriptor.
- The playback engine: a worker thread owns the decode source and feeds an AAudio realtime callback
  (`src/engine.rs`, `src/engine_worker.rs`, `src/output.rs`).
- The track fingerprint used as the analysis cache key (`src/fingerprint.rs`), computed by the shared
  `truepeak-core` crate so the desktop app and this crate agree on the value.
- The true-peak service and its cache: one opaque handle held by Kotlin (`src/service.rs`, `src/truepeak.rs`,
  `src/peak_cache.rs`), which drives the shared policy resolver of `truepeak-core`.
- A decode-throughput benchmark exposed to Kotlin (`src/bench.rs`).
- The JNI glue (`src/lib.rs`, `src/jni_call.rs`): every `Java_dev_monochromatic_..._native...` function is a method of
  the Kotlin `NativeBridge` class, found by its mangled name.

## Build

The crate cross-compiles for `arm64-v8a` (real devices) and `x86_64` (emulators) at API level 26 with `cargo-ndk`.
Run the tasks from `package/music-player/android-app`:

- `mise run //package/music-player/android-app:build:native` writes the `.so` files into `app/src/main/jniLibs`.
- `mise run //package/music-player/android-app:build` and `:build:release` rebuild it first.

Run `prepare:android` once on a new machine to provision the NDK that `build:native` derives `ANDROID_NDK_HOME` from.
`opus` bundles libopus and builds it from source with cmake.

## Lint and test

- `mise run //package/music-player/android-app:lint:clippy` runs Clippy over every Android-native target for both ABIs;
  `:format:clippy` applies its machine-applicable suggestions.
- The crate has no Rust unit tests of its own. Its behavior is covered through the Kotlin side (`:test:unit`) and the
  instrumented tests (`:test:instrumented`).
  The shared logic it calls is tested in `package/music-player/truepeak-core`.
