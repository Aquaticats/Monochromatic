//! The structural boundary of the standards review: the gateway client (`ureq`, TLS, the network) is reachable only
//! from the review run that the resident service calls, never from the wrapper's forwarded-command or commit-client
//! paths. The test reads the crate's own source, so a new `use` on a client path fails it before any binary is built.

use std::path::PathBuf;

/// The one file that may name the HTTP library.
const CLIENT_FILE: &str = "standards_review_client.rs";

/// The module names that carry the network client, the run that calls it, and the service handler that calls the run.
const SERVICE_ONLY_MODULES: [&str; 3] = ["standards_review_client", "standards_review_run", "service_review"];

/// Files outside the client path that may name the service-only modules: their own files and tests, the module
/// declarations, the service's work consumer (which owns the handler that calls the review), and this test.
fn may_name_service_modules(file: &str) -> bool {
    return file == "lib.rs"
        || file == "standards_review_boundary_tests.rs"
        || file.starts_with("service_work")
        || SERVICE_ONLY_MODULES.iter().any(|module| return file.starts_with(module));
}

/// The source files of this crate that ship in the executable or its library: every `.rs` file of the native directory
/// with its text.
fn native_sources() -> Vec<(String, String)> {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
    let mut sources: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(directory).expect("the native source directory").flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".rs") {
            sources.push((name, std::fs::read_to_string(entry.path()).expect("a source file")));
        }
    }
    sources.sort();
    return sources;
}

/// The violations in a set of sources: a file that names the HTTP library but is not the client, and a file that names
/// a service-only module but is not allowed to.
fn violations(sources: &[(String, String)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (file, text) in sources {
        let client_tests: bool = file.starts_with("standards_review_client");
        if text.contains("ureq") && !client_tests && file != "standards_review_boundary_tests.rs" {
            found.push(format!("{file} names the HTTP library"));
        }
        // The credential type is shared with the client path on purpose; only the handler module is service-only.
        let scanned: String = text.replace("service_review_key", "");
        if !may_name_service_modules(file) && SERVICE_ONLY_MODULES.iter().any(|module| return scanned.contains(module)) {
            found.push(format!("{file} names a service-only module"));
        }
    }
    return found;
}

/// The crate's real sources keep the client behind the service-only modules.
#[test]
fn the_network_client_is_reachable_only_from_the_service_run() {
    let sources: Vec<(String, String)> = native_sources();
    assert_eq!(violations(sources.as_slice()), Vec::<String>::new());
}

/// The scan is able to fail: a positive control on the real tree, and planted violations on a synthetic one.
#[test]
fn the_scan_sees_the_client_and_flags_a_planted_import() {
    let sources: Vec<(String, String)> = native_sources();
    let client: &(String, String) = sources.iter().find(|(file, _)| return file == CLIENT_FILE).expect("the client file");
    assert!(client.1.contains("ureq"), "the scan must see the library where it is used");
    let client_path_files: Vec<&str> = vec!["main.rs", "entry.rs", "forwarding.rs", "commit_command.rs"];
    for file in client_path_files {
        assert!(sources.iter().any(|(name, _)| return name == file), "{file} is part of the scanned tree");
    }
    let mut planted: Vec<(String, String)> = sources.clone();
    planted.push((String::from("forwarding_extra.rs"), String::from("use super::standards_review_client::post_decision;")));
    planted.push((String::from("entry_extra.rs"), String::from("let agent = ureq::Agent::new_with_defaults();")));
    let found: Vec<String> = violations(planted.as_slice());
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found.iter().any(|line| return line.starts_with("forwarding_extra.rs")));
    assert!(found.iter().any(|line| return line.starts_with("entry_extra.rs")));
}
