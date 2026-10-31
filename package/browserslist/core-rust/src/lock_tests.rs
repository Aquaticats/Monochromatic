//! What:
//!  Tests for lock parsing, validation and rendering.
//! Why:
//!  The lock is the trust anchor for every download, so each malformed shape must be refused with a message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('DataLock', () => { /* round trip and refusals */ });
//! ```

use crate::integrity::integrity_of;
use crate::lock::DataLock;

/// Build lock text with chosen pieces.
fn lock_text(version: &str, integrity: &str, esr: &str, extra: &str) -> String {
    return format!(
        "{{\"lockVersion\":1,\"caniuse-lite\":{{\"version\":\"{version}\",\"integrity\":\"{integrity}\"}},\
         \"node-releases\":{{\"version\":\"2.0.61\",\"integrity\":\"{integrity}\"}},\"firefoxEsr\":\"{esr}\"{extra}}}"
    );
}

/// A valid lock round-trips through rendering and back, with sorted keys and a final newline.
#[test]
fn round_trip() {
    let integrity = integrity_of(b"x");
    let lock = DataLock::from_json(&lock_text("1.0.30001820", &integrity, "153.6.0esr", "")).expect("parses");
    assert_eq!(lock.caniuse_lite.version, "1.0.30001820");
    assert_eq!(lock.node_releases.name, "node-releases");
    assert_eq!(lock.firefox_esr, "153.6.0esr");
    let text = lock.to_json();
    assert_eq!(text.ends_with("}\n"), true);
    assert_eq!(text.find("\"caniuse-lite\"") < text.find("\"firefoxEsr\""), true);
    assert_eq!(text.find("\"firefoxEsr\"") < text.find("\"lockVersion\""), true);
    assert_eq!(text.find("\"lockVersion\"") < text.find("\"node-releases\""), true);
    assert_eq!(DataLock::from_json(&text).expect("re-parses"), lock);
}

/// Every malformed lock fails with the lock named.
#[test]
fn refuses_malformed_locks() {
    let good = integrity_of(b"x");
    let mut cases: Vec<(String, &str)> = vec![
        (String::new(), "not valid JSON"),
        ("[]".to_string(), "not an object"),
        ("{}".to_string(), "exactly"),
        (lock_text("1.0.0", &good, "153.6.0esr", ",\"extra\":1"), "exactly"),
        (lock_text("1.0.0", &good, "153esr", ""), "<major>.<minor>.<patch>esr"),
        (lock_text("1.0.0", "sha1-x", "153.6.0esr", ""), "sha512-<base64>"),
        (lock_text("../x", &good, "153.6.0esr", ""), "outside letters"),
        (lock_text("", &good, "153.6.0esr", ""), "outside letters"),
        (lock_text("1.0.0", &good, "153.6.0esr", "").replace("\"lockVersion\":1", "\"lockVersion\":2"), "lockVersion must be 1"),
        (lock_text("1.0.0", &good, "153.6.0esr", "").replace("\"firefoxEsr\":\"153.6.0esr\"", "\"firefoxEsr\":153"), "firefoxEsr"),
        (lock_text("1.0.0", &good, "153.6.0esr", "").replace("\"version\":\"1.0.0\",", ""), "exactly"),
        (lock_text("1.0.0", &good, "153.6.0esr", "").replace("\"version\":\"1.0.0\"", "\"vers\":\"1.0.0\""), "needs string"),
        (lock_text("1.0.0", &good, "153.6.0esr", "").replace("\"caniuse-lite\":{", "\"caniuse-lite\":[{").replace("},\"node-releases\"", "}],\"node-releases\""), "not an object"),
    ];
    cases.push(("{\"lockVersion\":1}".to_string(), "exactly"));
    for (text, wanted) in cases {
        let message = DataLock::from_json(&text).expect_err("refused").to_string();
        assert_eq!(message.contains(wanted), true, "{text}: {message}");
    }
}
