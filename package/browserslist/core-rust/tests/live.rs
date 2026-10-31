// Opt-in live tests against the real npm registry and Mozilla product-details.
//
// They are ignored by default so the suite stays offline; run them with
// `mise run //package/browserslist/core-rust:test:live`.
//
// In TS you'd write (pseudocode):
//   test.skipIf(!process.env.LIVE)('resolves from the real registry', async () => { /* ... */ });

use monochromatic_browserslist::source::NPM_REGISTRY;
use monochromatic_browserslist::update::{MOZILLA_PRODUCT_DETAILS, latest_firefox_esr, latest_package};
use monochromatic_browserslist::{DataLock, DataSource, resolve_browserslistrc};

// The committed lock.
const LOCK: &str = include_str!("../browserslist-data.lock.json");
// The generated oracle answers.
const EXPECTED: &str = include_str!("../conformance/expected.json");

// A fresh cache fetches the pinned tarballs from npm, verifies them against the lock, and gives the npm answer.
#[test]
#[ignore = "live network: fetches the pinned tarballs from registry.npmjs.org"]
fn fresh_cache_resolves_the_repository_file_from_npm() {
    let lock = DataLock::from_json(LOCK).expect("lock parses");
    let scratch = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("scratch directory");
    let source = DataSource { cache_dir: scratch.path().join("cache"), registry_url: NPM_REGISTRY.to_string() };
    let expected: serde_json::Value = serde_json::from_str(EXPECTED).expect("expected.json parses");
    let case = &expected["cases"][0];
    let actual = resolve_browserslistrc(case["config"].as_str().expect("config"), &lock, &source).expect("resolves");
    let wanted: Vec<String> = case["npm"]["result"]
        .as_array()
        .expect("result")
        .iter()
        .map(|entry| return entry.as_str().expect("string").to_string())
        .collect();
    assert_eq!(actual, wanted);
}

// The registry and Mozilla still answer in the shapes the update step reads.
#[test]
#[ignore = "live network: reads registry.npmjs.org and product-details.mozilla.org"]
fn update_inputs_have_the_expected_shape() {
    let package = latest_package(NPM_REGISTRY, "caniuse-lite").expect("registry answers");
    assert_eq!(package.name, "caniuse-lite");
    assert!(package.integrity.starts_with("sha512-"));
    let esr = latest_firefox_esr(MOZILLA_PRODUCT_DETAILS).expect("Mozilla answers in the expected shape");
    assert!(esr.ends_with("esr"));
}
