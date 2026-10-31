//! What:
//!  Tests for the public entry point:
//! end to end through lock,
//! cache,
//! fake registry,
//! resolution and rendering.
//! Why:
//!  These prove the engine-facing call refuses unsupported input before any network use and renders the exact file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('resolveBrowserslistrc', () => { /* end to end, refusals touch no network */ });
//! ```

use crate::api::{render_browserslistrc, resolve_browserslistrc};
use crate::fixture::{fixture_lock, fixture_registry, scratch};
use crate::source::DataSource;

/// The seven repository lines.
const REPOSITORY_RC: &str = "last 1 Chrome versions\nlast 1 ChromeAndroid versions\nlast 1 Android versions\nFirefox ESR\nlast 1 FirefoxAndroid versions\nlast 1 node major versions\nnot dead\n";

/// Cold-cache resolution fetches once,
/// warm resolution fetches nothing,
/// and both give the same answer.
#[test]
fn end_to_end_with_a_fake_registry() {
    let registry = fixture_registry();
    let directory = scratch();
    let source = DataSource { cache_dir: directory.path().join("cache"), registry_url: registry.url() };
    let lock = fixture_lock();
    let first = resolve_browserslistrc(REPOSITORY_RC, &lock, &source).expect("resolves");
    assert_eq!(registry.requests().len(), 2);
    let second = resolve_browserslistrc(REPOSITORY_RC, &lock, &source).expect("resolves again");
    assert_eq!(registry.requests().len(), 2);
    assert_eq!(first, second);
    assert_eq!(first[..5].to_vec(), vec!["and_chr 156", "and_ff 159", "android 156", "chrome 156", "firefox 153"]);
}

/// The rendered text is the JSON array with two-space indentation and a final newline.
#[test]
fn renders_the_file_content() {
    let registry = fixture_registry();
    let directory = scratch();
    let source = DataSource { cache_dir: directory.path().join("cache"), registry_url: registry.url() };
    let text = render_browserslistrc("last 1 Chrome versions\nFirefox ESR\n", &fixture_lock(), &source).expect("renders");
    assert_eq!(text, "[\n  \"chrome 156\",\n  \"firefox 153\"\n]\n");
    let empty = render_browserslistrc("# nothing\n", &fixture_lock(), &source).expect("renders");
    assert_eq!(empty, "[]\n");
}

/// Unsupported queries and config features fail before the cache or the network is touched.
#[test]
fn refusals_touch_no_network() {
    let registry = fixture_registry();
    let directory = scratch();
    let source = DataSource { cache_dir: directory.path().join("cache"), registry_url: registry.url() };
    for text in ["last 2 Chrome versions\n", "defaults\n", "[production]\nnot dead\n", "a, b\n", "not dead\r\n", "Firefox ESR\nsupports es6\n"] {
        let message = resolve_browserslistrc(text, &fixture_lock(), &source).expect_err("refused").to_string();
        assert!(message.starts_with("not implemented: "), "{text:?}: {message}");
    }
    assert_eq!(registry.requests().len(), 0);
    assert!(!source.cache_dir.exists());
}

/// A first-position `not dead` is browserslist's own error (after the data loads).
#[test]
fn not_dead_first_is_reported() {
    let registry = fixture_registry();
    let directory = scratch();
    let source = DataSource { cache_dir: directory.path().join("cache"), registry_url: registry.url() };
    let message = resolve_browserslistrc("not dead\n", &fixture_lock(), &source).expect_err("rejected").to_string();
    assert_eq!(message, "Write any browsers query (for instance, `defaults`) before `not dead`");
}
