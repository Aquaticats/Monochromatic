// Conformance of the Rust resolver against npm browserslist 4.29.5.
//
// `conformance/expected.json` holds what the real package answered, inside a container, for every case of
// `conformance/cases.json`, over the same caniuse-lite and node-releases revision as the committed fixtures and the
// lock. Regenerate it with `mise run //package/browserslist/core-rust:conformance:regenerate`.
//
// In TS you'd write (pseudocode):
//   for (const testCase of expected.cases) expect(resolve(testCase.config, data)).toEqual(testCase.npm.result);

use monochromatic_browserslist::{BrowserData, BrowserslistError, DataFiles, DataLock, resolve_config_text};
use serde_json::Value;

// The generated oracle answers.
const EXPECTED: &str = include_str!("../conformance/expected.json");
// The committed lock the revision must agree with.
const LOCK: &str = include_str!("../browserslist-data.lock.json");
// The fixture data files regenerated together with `expected.json`.
const AGENTS_JS: &str = include_str!("../fixtures/data/agents.js.txt");
const BROWSERS_JS: &str = include_str!("../fixtures/data/browsers.js.txt");
const BROWSER_VERSIONS_JS: &str = include_str!("../fixtures/data/browserVersions.js.txt");
const ENVS_JSON: &str = include_str!("../fixtures/data/envs.json.txt");

// Parse the generated answers.
fn expected() -> Value {
    return serde_json::from_str(EXPECTED).expect("expected.json parses");
}

// Build data from the fixtures with a chosen ESR value and an optional Node.js version override.
fn data_with(firefox_esr: &str, node_versions: Option<Vec<String>>) -> BrowserData {
    let mut data = BrowserData::from_files(DataFiles {
        agents_js: AGENTS_JS,
        browsers_js: BROWSERS_JS,
        browser_versions_js: BROWSER_VERSIONS_JS,
        envs_json: ENVS_JSON,
        firefox_esr,
    })
    .expect("fixtures decode");
    if let Some(versions) = node_versions {
        data.node_versions = versions;
    }
    return data;
}

// The strings of a JSON array.
fn strings(value: &Value) -> Vec<String> {
    return value
        .as_array()
        .expect("an array")
        .iter()
        .map(|item| return item.as_str().expect("a string").to_string())
        .collect();
}

// Check one case against the Rust resolver; the error names the case and what differed.
fn check_case(case: &Value, locked_esr: &str, npm_esr: &str) -> Result<(), String> {
    let name = case["name"].as_str().expect("name");
    let config = case["config"].as_str().expect("config");
    let supported = case["supported"].as_bool().expect("supported");
    let node_versions = case.get("nodeVersions").map(strings);
    let data = data_with(locked_esr, node_versions);
    let outcome = resolve_config_text(config, &data);
    if !supported {
        return match outcome {
            Err(BrowserslistError::NotImplemented { .. }) => Ok(()),
            other => Err(format!("{name}: expected a not-implemented refusal, got {other:?}")),
        };
    }
    let locked_major = locked_esr.split('.').next().expect("a major");
    let esr_entry = format!("firefox {locked_major}");
    if let Some(message) = case["npm"].get("error") {
        let wanted = message.as_str().expect("message");
        return match outcome {
            Err(error) if error.to_string() == wanted => Ok(()),
            other => Err(format!("{name}: expected the npm error {wanted:?}, got {other:?}")),
        };
    }
    // The ESR entry is compared against the locked Mozilla value, not npm's constant (the user accepted that).
    let wanted: Vec<String> = strings(&case["npm"]["result"])
        .into_iter()
        .map(|entry| return if entry == npm_esr { esr_entry.clone() } else { entry })
        .collect();
    return match outcome {
        Ok(actual) if actual == wanted => Ok(()),
        other => Err(format!("{name}: expected {wanted:?}, got {other:?}")),
    };
}

// The oracle ran the revision the lock pins, and the fixtures were regenerated with it.
#[test]
fn expected_json_matches_the_lock() {
    let document = expected();
    let lock = DataLock::from_json(LOCK).expect("lock parses");
    assert_eq!(document["browserslist"], "4.29.5");
    assert_eq!(document["caniuseLite"].as_str(), Some(lock.caniuse_lite.version.as_str()));
    assert_eq!(document["nodeReleases"].as_str(), Some(lock.node_releases.version.as_str()));
}

// Every case, over the seven lines together and each alone, node variants, and the refusals.
#[test]
fn every_case_conforms() {
    let document = expected();
    let lock = DataLock::from_json(LOCK).expect("lock parses");
    let npm_esr = document["npmFirefoxEsr"].as_str().expect("npm ESR entry");
    let cases = document["cases"].as_array().expect("cases");
    assert!(cases.len() >= 50, "only {} cases", cases.len());
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|case| return check_case(case, &lock.firefox_esr, npm_esr).err())
        .collect();
    assert_eq!(failures, Vec::<String>::new());
}

// The suite exercises both answers and refusals, and npm's own error text.
#[test]
fn cases_cover_results_errors_and_refusals() {
    let document = expected();
    let cases = document["cases"].as_array().expect("cases");
    let supported_results = cases.iter().filter(|case| return case["supported"] == true && case["npm"].get("result").is_some()).count();
    let supported_errors = cases.iter().filter(|case| return case["supported"] == true && case["npm"].get("error").is_some()).count();
    let refusals = cases.iter().filter(|case| return case["supported"] == false).count();
    assert!(supported_results >= 15);
    assert!(supported_errors >= 2);
    assert!(refusals >= 25);
}

// Positive control: a wrong expectation must fail the comparison, so the harness can see a difference.
#[test]
fn control_a_wrong_expectation_fails() {
    let document = expected();
    let lock = DataLock::from_json(LOCK).expect("lock parses");
    let npm_esr = document["npmFirefoxEsr"].as_str().expect("npm ESR entry");
    let mut case = document["cases"][0].clone();
    assert!(check_case(&case, &lock.firefox_esr, npm_esr).is_ok());
    case["npm"]["result"][3] = Value::from("chrome 155");
    assert!(check_case(&case, &lock.firefox_esr, npm_esr).is_err());
    // A supported case that npm answers with an error must fail when the Rust side answers instead.
    let mut refused = document["cases"][0].clone();
    refused["supported"] = Value::from(false);
    assert!(check_case(&refused, &lock.firefox_esr, npm_esr).is_err());
}

// The ESR substitution: with a different locked major the only difference from npm is the ESR entry.
#[test]
fn esr_comes_from_the_lock_not_from_npm() {
    let document = expected();
    let npm_esr = document["npmFirefoxEsr"].as_str().expect("npm ESR entry");
    let case = &document["cases"][0];
    assert_eq!(check_case(case, "160.2.0esr", npm_esr), Ok(()));
    let config = case["config"].as_str().expect("config");
    let ours = resolve_config_text(config, &data_with("160.2.0esr", None)).expect("resolves");
    assert!(ours.contains(&"firefox 160".to_string()));
    assert!(!ours.contains(&npm_esr.to_string()));
}
