//! Link Android's liblog so the logcat tracing layer resolves its log symbols.
//!
//! `paranoid-android` reaches logcat through `ndk-sys` 0.5,
//!  whose `__android_log_*` FFI
//! declarations carry no `#[link]` directive,
//!  and neither the Rust android target nor
//! cargo-ndk requests `-llog`.
//!  liblog ships with the Android platform,
//!  so we ask the
//! linker for it here (guarded to android targets,
//!  though this cdylib only builds there).
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        println!("cargo:rustc-link-lib=dylib=log");
    }
}
