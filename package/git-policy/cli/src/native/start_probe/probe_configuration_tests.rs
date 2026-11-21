//! What:
//!  Controls of the two names the root-owned configuration supplies:
//!  a document with comments and both keys,
//!  a missing key,
//!  a value that is no text,
//!  a name the rule refuses,
//!  and the checkout's absolute-path rule.
//! Why:
//!  A configuration the setup writes must be read the same way every boot,
//!  and a boot whose configuration names nothing must record the arm that did
//!  not run rather than a pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(sourceFromConfiguration('{"gitWorktreeProbeDataset":"pool/ds"}')).toEqual({ dataset: 'pool/ds' });
//! ```

use super::{CheckoutRefusal, SourceRefusal, checkout_from_configuration, read_configuration, source_from_configuration};
use crate::testing::test_support::fixture;
use std::path::{Path, PathBuf};

/// A document with comments and both keys answers the source and the
///  checkout.
#[test]
fn a_document_with_comments_answers_both_names() {
    let text: &str = "{\n  // the probe's own dataset\n  \"gitWorktreeProbeDataset\": \"zpcachyos/nosnap/git-worktree-probe\",\n  \"gitWorktreeCheckout\": \"/home/user/Monochromatic\",\n}\n";
    let path: PathBuf = PathBuf::from("/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc");
    let source = source_from_configuration(text, path.as_path()).expect("the source is named");
    assert_eq!(source.snapshot_name(), "zpcachyos/nosnap/git-worktree-probe@ready");
    assert_eq!(checkout_from_configuration(text, path.as_path()), Ok(PathBuf::from("/home/user/Monochromatic")));
}

/// A document without the key names no source,
/// and one without the checkout names no checkout.
#[test]
fn a_missing_key_is_named_by_its_cause() {
    let path: PathBuf = PathBuf::from("/etc/handler.jsonc");
    assert!(matches!(source_from_configuration("{}", path.as_path()), Err(SourceRefusal::NameAbsent { .. })));
    assert_eq!(checkout_from_configuration("{}", path.as_path()), Err(CheckoutRefusal::AbsentOrNotText));
}

/// A value that is no text is refused rather than read as a name.
#[test]
fn a_value_that_is_no_text_is_refused() {
    let path: PathBuf = PathBuf::from("/etc/handler.jsonc");
    assert!(matches!(source_from_configuration("{\"gitWorktreeProbeDataset\": 7}", path.as_path()), Err(SourceRefusal::NameAbsent { .. })));
    assert_eq!(checkout_from_configuration("{\"gitWorktreeCheckout\": []}", path.as_path()), Err(CheckoutRefusal::AbsentOrNotText));
}

/// A name the rule refuses is refused with the rule's own cause,
/// and its word reaches the record.
#[test]
fn a_refused_name_carries_its_cause() {
    let path: PathBuf = PathBuf::from("/etc/handler.jsonc");
    let refusal = source_from_configuration("{\"gitWorktreeProbeDataset\": \"pool/ds@snap\"}", path.as_path()).expect_err("the name is refused");
    assert_eq!(refusal.word(), "name-refused");
    assert!(refusal.message().contains("pool/ds@snap"));
}

/// A checkout that is not absolute is refused.
#[test]
fn a_relative_checkout_is_refused() {
    assert_eq!(checkout_from_configuration("{\"gitWorktreeCheckout\": \"relative/path\"}", Path::new("/etc/handler.jsonc")), Err(CheckoutRefusal::NotAbsolute));
}

/// A document that is not JSONC is its own cause.
#[test]
fn a_document_that_does_not_parse_is_its_own_cause() {
    let path: PathBuf = PathBuf::from("/etc/handler.jsonc");
    assert!(matches!(source_from_configuration("{", path.as_path()), Err(SourceRefusal::ConfigurationUnparsable { .. })));
    assert_eq!(checkout_from_configuration("{", path.as_path()), Err(CheckoutRefusal::Unparsable));
}

/// An absent file is the missing-configuration cause,
/// never a name the parse refused.
#[test]
fn an_absent_file_is_its_own_cause() {
    let root: PathBuf = fixture("start-probe-config-absent");
    let path: PathBuf = root.join("no-such.jsonc");
    match read_configuration(path.as_path()) {
        Err(SourceRefusal::ConfigurationMissing { path: named }) => assert_eq!(named, path),
        other => panic!("{other:?}"),
    }
}
