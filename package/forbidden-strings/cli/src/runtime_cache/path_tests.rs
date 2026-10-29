//! Cross-platform cache-root and content-key tests.
//!
//! The Windows target branch validates paths byte-explicitly,
//!  so its assertions run on every host.
//! The XDG and macOS branches validate with the host's `Path::is_absolute`,
//!  which follows Windows rules on a Windows host;
//! production never selects those branches there,
//!  so their assertions run on Unix hosts only.

use super::{
    cache_location, resolve_cache_root, source_digest, CacheEnvironment, CacheRootError, HostPlatform,
};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

/// Explicit absolute override wins under Unix target semantics.
///
/// What:
///  `#[cfg(unix)]` compiles this test only for Unix hosts.
/// Why:
///  `/private/cache` is absolute only under Unix path rules,
///  which the Unix branches borrow from the host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test.skipIf(process.platform === 'win32')('absolute unix override wins', () => { ... });
/// ```
#[cfg(unix)]
#[test]
fn absolute_unix_override_wins() {
    let unix_environment = CacheEnvironment {
        override_root: Some(OsString::from("/private/cache")),
        ..CacheEnvironment::default()
    };
    for platform in [HostPlatform::XdgUnix, HostPlatform::Macos] {
        assert_eq!(
            resolve_cache_root(&unix_environment, platform).expect("absolute override"),
            PathBuf::from("/private/cache"),
        );
    }
}

/// Explicit absolute override wins under Windows target semantics on every host.
#[test]
fn absolute_windows_override_wins() {
    let windows_environment = CacheEnvironment {
        override_root: Some(OsString::from("C:\\private\\cache")),
        ..CacheEnvironment::default()
    };
    assert_eq!(
        resolve_cache_root(&windows_environment, HostPlatform::Windows)
            .expect("Windows override"),
        PathBuf::from("C:\\private\\cache"),
    );
}

/// Relative explicit override is configuration error rather than cwd-relative cache.
#[test]
fn relative_override_is_rejected() {
    let environment = CacheEnvironment {
        override_root: Some(OsString::from("relative/cache")),
        ..CacheEnvironment::default()
    };
    assert_eq!(
        resolve_cache_root(&environment, HostPlatform::XdgUnix),
        Err(CacheRootError::InvalidOverride),
    );
}

/// XDG absolute value wins and relative value falls back to absolute HOME cache,
///  under Unix host path rules.
#[cfg(unix)]
#[test]
fn xdg_resolution_follows_base_directory_spec() {
    let absolute = CacheEnvironment {
        xdg_cache_home: Some(OsString::from("/xdg/cache")),
        home: Some(OsString::from("/home/user")),
        ..CacheEnvironment::default()
    };
    assert_eq!(
        resolve_cache_root(&absolute, HostPlatform::XdgUnix).expect("XDG root"),
        PathBuf::from("/xdg/cache"),
    );

    let relative = CacheEnvironment {
        xdg_cache_home: Some(OsString::from("relative")),
        home: Some(OsString::from("/home/user")),
        ..CacheEnvironment::default()
    };
    assert_eq!(
        resolve_cache_root(&relative, HostPlatform::XdgUnix).expect("HOME fallback"),
        PathBuf::from("/home/user/.cache"),
    );
}

/// macOS uses the selected HOME root,
///  under Unix host path rules.
#[cfg(unix)]
#[test]
fn macos_native_root_is_derived() {
    let environment = CacheEnvironment {
        home: Some(OsString::from("/Users/alice")),
        local_app_data: Some(OsString::from("C:\\Users\\alice\\AppData\\Local")),
        ..CacheEnvironment::default()
    };
    assert_eq!(
        resolve_cache_root(&environment, HostPlatform::Macos).expect("macOS root"),
        PathBuf::from("/Users/alice/Library/Caches"),
    );
}

/// Windows uses the selected local application-data root on every host,
///  ignoring HOME.
#[test]
fn windows_native_root_is_derived() {
    let environment = CacheEnvironment {
        home: Some(OsString::from("/Users/alice")),
        local_app_data: Some(OsString::from("C:\\Users\\alice\\AppData\\Local")),
        ..CacheEnvironment::default()
    };
    assert_eq!(
        resolve_cache_root(&environment, HostPlatform::Windows).expect("Windows root"),
        PathBuf::from("C:\\Users\\alice\\AppData\\Local"),
    );
}

/// Missing native root reports unavailable without exposing environment values.
#[test]
fn missing_native_root_is_unavailable() {
    assert_eq!(
        resolve_cache_root(&CacheEnvironment::default(), HostPlatform::XdgUnix),
        Err(CacheRootError::Unavailable),
    );
}

/// What:
///  Returns the UTF-8 text of one normal path component,
///  or fails the test for any other component kind.
/// Why:
///  The artifact layout is asserted as a component sequence,
///  so the host's separator spelling never matters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function normalName(component: PathComponent): string { assert(component.kind === 'normal'); return component.text; }
/// ```
fn normal_name(component: Component<'_>) -> String {
    // What: `if let Component::Normal(name) = component` extracts the name only from an ordinary directory or file component.
    // Why: A root, prefix or navigation marker in the artifact layout would be a layout defect.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (component.kind === 'normal') return component.text;
    // ```
    if let Component::Normal(name) = component {
        // `.to_str()` borrows the name as UTF-8 text; `.expect` fails the test on non-UTF-8 bytes; `.to_string()` copies it.
        return name.to_str().expect("UTF-8 artifact component").to_string();
    }
    panic!("artifact layout contains a non-name component: {component:?}");
}

/// Exact source bytes select stable lowercase content-addressed artifact path.
#[test]
fn source_bytes_select_content_addressed_path() {
    let first = source_digest(b"alpha\n").expect("digest");
    let repeated = source_digest(b"alpha\n").expect("digest");
    let changed = source_digest(b"alpha").expect("digest");
    assert_eq!(first, repeated);
    assert_ne!(first, changed);

    let root: PathBuf = PathBuf::from("/cache");
    let location = cache_location(&root, first);
    // What: `strip_prefix` returns the path below `root`, failing if the artifact is not under it.
    // Why: Components below the root are compared one by one instead of a rendered string,
    // whose separator is `/` on Unix and `\` on Windows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const below: string[] = path.relative(root, location.artifactPath).split(path.sep);
    // ```
    let below: &Path = location.artifact_path.strip_prefix(&root).expect("artifact under root");
    let mut names: Vec<String> = Vec::new();
    for component in below.components() {
        names.push(normal_name(component));
    }
    assert_eq!(names.len(), 5, "application, version, platform, digest and filename: {names:?}");
    assert_eq!(names[0], "forbidden-strings");
    // Independent spellings of the version and compile-target partitions, not the helpers that build them.
    assert_eq!(names[1], format!("v{}", env!("CARGO_PKG_VERSION")));
    assert_eq!(names[2], format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH));
    assert_eq!(names[4], "rules.bin");
    let digest_component: &str = names[3].as_str();
    assert_eq!(digest_component.len(), 64);
    for character in digest_component.chars() {
        assert!(character.is_ascii_digit() || ('a'..='f').contains(&character), "lowercase hex digest: {digest_component}");
    }
}
