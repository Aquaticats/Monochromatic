//! The table this build embeds:
//!  every digest holds,
//!  the table is sorted,
//!  every listed grammar and
//! its notices are present,
//!  and every language on a bundled grammar compiles from the table alone,
//! unpacking into a disposable cache,
//!  with a shadowing user Helix query present.

use crate::embedded_runtime::EMBEDDED_RUNTIME;
use helix_core::syntax::config::Configuration;
use ide_app::{
    runtime::{self, RuntimeSource},
    syntax::{SyntaxEngine, provisioned_grammars_from},
};
use std::{collections::HashSet, fs, path::Path, process::Command};

/// Set in the child process,
///  which performs the compile check instead of starting another child.
const CHILD: &str = "IDE_EMBEDDED_RUNTIME_CHILD";

/// The grammar names the embedded manifest lists.
fn listed() -> HashSet<String> {
    let manifest = EMBEDDED_RUNTIME
        .required("runtime/manifest.json")
        .expect("embedded manifest");
    let text = std::str::from_utf8(manifest).expect("UTF-8 manifest");
    return provisioned_grammars_from(text, "embedded manifest", "rebuild").expect("manifest");
}

#[test]
fn every_embedded_file_matches_its_build_digest_and_the_table_is_sorted() {
    assert!(
        !EMBEDDED_RUNTIME.files.is_empty(),
        "the build embedded nothing"
    );
    for file in EMBEDDED_RUNTIME.files {
        EMBEDDED_RUNTIME
            .verified(file.path)
            .expect("intact embedded file")
            .expect("listed file is found");
    }
    for pair in EMBEDDED_RUNTIME.files.windows(2) {
        assert!(
            pair[0].path < pair[1].path,
            "{} is not before {}",
            pair[0].path,
            pair[1].path
        );
    }
    for name in listed() {
        EMBEDDED_RUNTIME
            .required(&format!("runtime/grammars/{name}.so"))
            .expect("listed grammar library");
        let prefix = format!("runtime/licenses/{name}/");
        let mut noticed = false;
        for file in EMBEDDED_RUNTIME.files.iter() {
            if file.path.starts_with(&prefix) {
                noticed = true;
            }
        }
        assert!(noticed, "no license notice embedded for {name}");
    }
    for path in [
        "runtime/Helix-LICENSE",
        "LICENSES/LGPL-3.0-or-later.txt",
        "LICENSES/GPL-3.0-or-later.txt",
        "LICENSES/font/Inter-LICENSE.txt",
        "LICENSES/font/JetBrainsMono-OFL.txt",
    ] {
        EMBEDDED_RUNTIME
            .required(path)
            .expect("embedded license text");
    }
}

#[test]
fn every_language_on_a_bundled_grammar_compiles_from_the_embedded_table_alone() {
    if std::env::var_os(CHILD).is_some() {
        runtime::install(RuntimeSource::Embedded(&EMBEDDED_RUNTIME));
        let engine = SyntaxEngine::new().expect("embedded language runtime");
        let names = listed();
        let config: Configuration = helix_loader::config::default_lang_config()
            .try_into()
            .expect("bundled language configuration");
        let mut covered: HashSet<String> = HashSet::new();
        let mut failures: Vec<String> = Vec::new();
        let mut checked = 0;
        for language in &config.language {
            let grammar = language.grammar.as_deref().unwrap_or(&language.language_id);
            if !names.contains(grammar) {
                continue;
            }
            covered.insert(grammar.to_string());
            match engine.compile_language(&language.language_id) {
                Ok(true) => checked += 1,
                Ok(false) => failures.push(format!("{}: not compiled", language.language_id)),
                Err(error) => failures.push(format!("{error:#}")),
            }
        }
        println!("languages compiled from the embedded table: {checked}");
        assert!(failures.is_empty(), "{failures:#?}");
        assert_eq!(
            covered, names,
            "a bundled grammar is used by no registry language"
        );
        return;
    }
    let directory = tempfile::tempdir().expect("disposable cache and configuration homes");
    let cache = directory.path().join("cache");
    let queries = directory.path().join("config/helix/runtime/queries/sql");
    fs::create_dir_all(&queries).expect("user Helix query folder");
    fs::write(queries.join("highlights.scm"), "").expect("empty shadowing query");
    let status = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "embedded_runtime_tests::every_language_on_a_bundled_grammar_compiles_from_the_embedded_table_alone",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD, "1")
        .env("XDG_CACHE_HOME", &cache)
        .env("XDG_CONFIG_HOME", directory.path().join("config"))
        .env_remove("HELIX_RUNTIME")
        .status()
        .expect("child test process");
    assert!(
        status.success(),
        "the embedded compile check failed in its child process: {status}"
    );
    let grammars = cache
        .join("monochromatic-ide/runtime")
        .join(EMBEDDED_RUNTIME.key)
        .join("grammars");
    let mut unpacked: HashSet<String> = HashSet::new();
    for entry in fs::read_dir(&grammars).expect("unpacked grammar folder") {
        let name = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        let path = grammars.join(&name);
        let library = name
            .strip_suffix(".so")
            .expect("only libraries remain")
            .to_string();
        let embedded = EMBEDDED_RUNTIME
            .required(&format!("runtime/grammars/{name}"))
            .expect("unpacked library is embedded");
        assert_eq!(
            fs::read(Path::new(&path)).expect("unpacked library"),
            embedded
        );
        unpacked.insert(library);
    }
    assert_eq!(
        unpacked,
        listed(),
        "the cache holds exactly the bundled libraries"
    );
}
