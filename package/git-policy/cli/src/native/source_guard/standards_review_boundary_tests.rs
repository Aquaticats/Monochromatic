//! The structural boundary of the standards review:
//! the gateway client (`hyper`,
//! TLS,
//! the network) is reachable only
//! from the review run that the resident service calls,
//! never from the wrapper's forwarded-command or commit-client
//! paths.
//! The test reads the crate's own source,
//! so a new `use` on a client path fails it before any binary is built.

use std::path::Path;

use crate::source_scan_tests::{CrateRoot, classified_sources, module_name, real_roots, source_root};

/// Filesystem-backed controls of this scan on nested layouts.
#[path = "standards_review_boundary_fixture_tests.rs"]
mod fixtures;

/// The one file that may name the HTTP library.
const CLIENT_FILE: &str = "standards_review/standards_review_client.rs";

/// The paths of the HTTP libraries the client is built on;
/// only the client file may name one (`hyper_rustls` and `http_body_util`
///  come with `hyper`,
/// and `rustls` is not listed because the wrapper's other code may read certificates).
const HTTP_LIBRARY_PATHS: [&str; 4] = ["hyper::", "hyper_util::", "hyper_rustls::", "http_body_util::"];

/// Whether `text` names one of the HTTP libraries by path.
fn names_http_library(text: &str) -> bool {
    return HTTP_LIBRARY_PATHS.iter().any(|path| { return text.contains(path); });
}

/// The module names that carry the network client,
/// the run that calls it,
/// and the service handler that calls the run.
const SERVICE_ONLY_MODULES: [&str; 4] =
    ["standards_review_client", "standards_review_client_name_lookup_on_the_blocking_pool", "standards_review_run", "service_review"];

/// The path prefixes of the files of those modules and their tests,
///  relative to the source root.
const SERVICE_ONLY_FILE_PREFIXES: [&str; 3] =
    ["standards_review/standards_review_client", "standards_review/standards_review_run", "service/service_review"];

/// Files outside the client path that may name the service-only modules:
/// their own files and tests,
/// the module
/// declarations,
/// the service's work consumer (which owns the handler that calls the review),
/// this test with its
/// fixture controls (`standards_review_boundary_*`),
/// which name the library and the modules on purpose,
/// and the rules of the scan that keeps blocked work off the runtime,
/// whose list of kept threads and pauses names the files it excuses,
/// a module of the review among them.
fn may_name_service_modules(file: &str) -> bool {
    return file == "lib.rs"
        || file.starts_with("source_guard/standards_review_boundary_")
        || file.starts_with("source_guard/blocking_code_off_the_runtime_rules_")
        || file.starts_with("landing/service_work")
        || SERVICE_ONLY_FILE_PREFIXES.iter().any(|prefix| return file.starts_with(prefix));
}

/// The source files of this crate that ship in the executable or its library,
/// or are compiled for its tests:
/// every
/// `.rs` file under `root` with its text,
/// keyed by its `/`-separated path relative to `root`.
/// The listing is the module
/// tree proven against a strict walk,
/// so a file in a subdirectory is read at its own path and a file the listing
/// could not account for is an error,
/// never a gap.
fn native_sources_in(root: &Path, roots: &[CrateRoot]) -> Result<Vec<(String, String)>, String> {
    let mut sources: Vec<(String, String)> = Vec::new();
    for file in classified_sources(root, roots)? {
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        sources.push((file.path, text));
    }
    return Ok(sources);
}

/// The sources of this crate's own native directory.
fn native_sources() -> Vec<(String, String)> {
    return native_sources_in(&source_root(), &real_roots()).expect("the crate's own sources");
}

/// The text without the bare `mod <service-only module>;` lines:
/// a module file registers its own members,
/// which is
/// declaring them and not using them.
fn without_declarations(text: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for line in text.lines() {
        let declares_service_module: bool =
            module_name(line).is_some_and(|name| return SERVICE_ONLY_MODULES.contains(&name));
        if !declares_service_module {
            kept.push(line);
        }
    }
    return kept.join("\n");
}

/// The violations in a set of sources:
/// a file that names the HTTP library but is not the client,
/// and a file that names
/// a service-only module but is not allowed to.
fn violations(sources: &[(String, String)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (file, text) in sources {
        let client_tests: bool = file.starts_with("standards_review/standards_review_client");
        if names_http_library(text) && !client_tests && !file.starts_with("source_guard/standards_review_boundary_") {
            found.push(format!("{file} names the HTTP library"));
        }
        // The credential type is shared with the client path on purpose; only the handler module is service-only.
        let scanned: String = without_declarations(text).replace("service_review_key", "");
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

/// The scan is able to fail:
/// a positive control on the real tree,
/// and planted violations on a synthetic one.
#[test]
fn the_scan_sees_the_client_and_flags_a_planted_import() {
    let sources: Vec<(String, String)> = native_sources();
    let client: &(String, String) = sources.iter().find(|(file, _)| return file == CLIENT_FILE).expect("the client file");
    assert!(names_http_library(client.1.as_str()), "the scan must see the library where it is used");
    let client_path_files: Vec<&str> = vec!["main.rs", "invocation/entry.rs", "invocation/forwarding.rs", "commit/commit_command.rs"];
    for file in client_path_files {
        assert!(sources.iter().any(|(name, _)| return name == file), "{file} is part of the scanned tree");
    }
    let mut planted: Vec<(String, String)> = sources.clone();
    planted.push((String::from("forwarding_extra.rs"), String::from("use super::standards_review_client::post_decision;")));
    planted.push((String::from("entry_extra.rs"), String::from("let client = hyper_util::client::legacy::Client::builder(executor);")));
    let found: Vec<String> = violations(planted.as_slice());
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found.iter().any(|line| return line.starts_with("forwarding_extra.rs")));
    assert!(found.iter().any(|line| return line.starts_with("entry_extra.rs")));
}
