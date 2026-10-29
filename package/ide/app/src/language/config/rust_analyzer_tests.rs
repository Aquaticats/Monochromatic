//! rust-analyzer is told to watch through the IDE,
//!  and merged definitions keep their other settings.

/// The registry under test and its unconfined setup.
use super::super::{LanguageSetup, Languages};
/// What:
///  `json_value!` builds a JSON value from literal syntax.
/// Why:
///  The expected settings are written as JSON.
///  The macro is `json!` with the one lint allowance the repository's `unwrap` ban needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const expected = { watcher: 'client' };
/// ```
use crate::json_value;

/// The settings Helix will send rust-analyzer for a registry built with `setup`.
fn settings(setup: LanguageSetup) -> serde_json::Value {
    let directory = tempfile::tempdir().expect("disposable project");
    let root = directory.path().canonicalize().expect("canonical root");
    let languages = Languages::new(&root, setup).expect("registry");
    let loader = languages.loader.load();
    let definition = loader
        .language_server_configs()
        .get("rust-analyzer")
        .expect("rust-analyzer definition");
    return definition.config.clone().expect("rust-analyzer settings");
}

/// Helix's own definition asks for server-side watching;
///  the IDE replaces it with client-side watching.
#[test]
fn rust_analyzer_watches_through_the_client() {
    let found = settings(LanguageSetup::unconfined());
    assert_eq!(found["files"]["watcher"], json_value!("client"));
}

/// A merged definition replaces the whole `files` table;
///  the watcher is still set,
///  and its siblings stay.
#[test]
fn merged_file_settings_keep_their_other_keys() {
    let setup = LanguageSetup {
        extra_languages: Some(
            "[language-server.rust-analyzer.config.files]\nexcludeDirs = [\"vendor\"]\n"
                .to_string(),
        ),
        ..LanguageSetup::unconfined()
    };
    let found = settings(setup);
    assert_eq!(
        found["files"],
        json_value!({ "excludeDirs": ["vendor"], "watcher": "client" })
    );
}
