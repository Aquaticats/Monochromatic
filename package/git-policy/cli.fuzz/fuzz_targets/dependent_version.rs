//! What: Every input through the dependent-version scanners: as the layout of a generated
//!       manifest, as `from`, `to` and raw manifest text, and as a package name and source.
//! Why: The generated view checks the rewrite against an independently built result; the
//!      raw views reach every scanner state, including unterminated literals and odd names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkGeneratedManifest(generatedManifest(data)); checkVersionEdit(...versionEditView(data)); checkImports(...importView(data)); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::dependent_version::{
    check_generated_manifest, check_imports, check_version_edit, generated_manifest, import_view,
    version_edit_view,
};
use libfuzzer_sys::fuzz_target;

// Run every view of every input.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_generated_manifest(&generated_manifest(data));
    let (from, to, text) = version_edit_view(data);
    let _ = check_version_edit(&text, &from, &to);
    let (name, source) = import_view(data);
    let _ = check_imports(&source, &name);
});
