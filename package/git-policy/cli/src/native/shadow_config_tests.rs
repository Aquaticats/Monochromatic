//! Controls for the shadow `config` file and the copied repository format.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::path::PathBuf;

/// Quoting escapes exactly the characters Git's config syntax needs and keeps the rest literal.
#[test]
fn quoting_escapes_only_what_git_requires() {
    assert_eq!(quote_git_config_value("/a \"b\""), r#""/a \"b\"""#);
    assert_eq!(quote_git_config_value("a\\b"), r#""a\\b""#);
    assert_eq!(quote_git_config_value("a\nb\tc"), r#""a\nb\tc""#);
    assert_eq!(quote_git_config_value("x ; # y "), r#""x ; # y ""#);
    assert_eq!(quote_git_config_value(""), r#""""#);
}

/// A repository without extensions writes the fixed layout with no extensions section.
#[test]
fn config_without_extensions_has_the_fixed_layout() {
    let format: RepositoryFormat = RepositoryFormat {
        version: String::from("0"),
        extensions: Vec::new(),
    };
    let text: String = format_shadow_config(
        &format,
        Path::new("/r/.git"),
        Path::new("/r"),
    );
    assert_eq!(
        text,
        concat!(
            "[core]\n",
            "\trepositoryformatversion = \"0\"\n",
            "[include]\n",
            "\tpath = \"/r/.git/config\"\n",
            "[core]\n",
            "\tworktree = \"/r\"\n",
            "[gc]\n",
            "\tauto = 0\n",
            "[maintenance]\n",
            "\tauto = false\n",
        )
    );
}

/// Extensions are copied under their own section, before the include.
#[test]
fn config_copies_extensions_before_the_include() {
    let format: RepositoryFormat = RepositoryFormat {
        version: String::from("1"),
        extensions: vec![(String::from("objectformat"), String::from("sha256"))],
    };
    let text: String = format_shadow_config(&format, Path::new("/r/.git"), Path::new("/r"));
    assert!(text.contains("[extensions]\n\tobjectformat = \"sha256\"\n[include]\n"), "{text}");
}

/// A sha1 repository reports version 0 and no extensions.
#[test]
fn sha1_repository_format_is_version_zero_without_extensions() {
    let root: PathBuf = fixture("shadow-config-sha1");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let format: RepositoryFormat =
        read_repository_format(&git_context(), repo.join(".git").as_path()).expect("format");
    assert_eq!(format.version, "0");
    assert!(format.extensions.is_empty());
    remove(root.as_path());
}

/// A sha256 repository copies its object format extension and version 1.
#[test]
fn sha256_repository_copies_its_object_format() {
    let root: PathBuf = fixture("shadow-config-sha256");
    git(root.as_path(), &["init", "--quiet", "--object-format=sha256", "repo"]);
    let repo: PathBuf = root.join("repo");
    let format: RepositoryFormat =
        read_repository_format(&git_context(), repo.join(".git").as_path()).expect("format");
    assert_eq!(format.version, "1");
    assert!(format
        .extensions
        .contains(&(String::from("objectformat"), String::from("sha256"))));
    remove(root.as_path());
}

/// A valueless extension reads as `true`, and a non-extension key is not copied.
#[test]
fn valueless_extension_reads_as_true_and_other_keys_are_skipped() {
    let root: PathBuf = fixture("shadow-config-valueless");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    std::fs::OpenOptions::new()
        .append(true)
        .open(common.join("config"))
        .and_then(|mut file| {
            use std::io::Write;
            return file.write_all(b"[extensions]\n\tworktreeConfig\n[user]\n\tname = Someone\n");
        })
        .expect("append");
    let format: RepositoryFormat =
        read_repository_format(&git_context(), common.as_path()).expect("format");
    assert!(format
        .extensions
        .contains(&(String::from("worktreeconfig"), String::from("true"))));
    assert!(format.extensions.iter().all(|(name, _)| { return !name.starts_with("user"); }));
    remove(root.as_path());
}

/// For each configuration the format from the snapshot equals the format from Git's `--file` listing.
#[test]
fn the_snapshot_format_equals_the_file_format() {
    let root: PathBuf = fixture("shadow-config-snapshot-format");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let context: GitContext = git_context();
    for text in [
        "[core]\n\trepositoryformatversion = 0\n",
        "[core]\n\trepositoryformatversion = 1\n[extensions]\n\tnoop = true\n",
        "[core]\n\trepositoryformatversion = 0\n\trepositoryformatversion = 1\n[extensions]\n\tworktreeConfig\n",
        "[core]\n\tRepositoryFormatVersion = 1\n[Extensions]\n\tNoop = yes\n\tworktreeConfig = false\n",
        "[user]\n\tname = Someone\n[core]\n\trepositoryformatversion = 1\n[extensions]\n\tnoop = a\n[extensions]\n\tnoop = b\n",
        "",
    ] {
        std::fs::write(common.join("config"), text).expect("config");
        let snapshot: ConfigSnapshot =
            crate::config_snapshot::read_config_snapshot(&context, repo.as_path()).expect("snapshot");
        let from_snapshot: RepositoryFormat = repository_format_from_snapshot(&snapshot).expect("no includes");
        let from_file: RepositoryFormat = read_repository_format(&context, common.as_path()).expect("format");
        assert_eq!(from_snapshot, from_file, "{text}");
    }
    remove(root.as_path());
}

/// Values that enter the snapshot from outside the common config file do not enter the format.
#[test]
fn entries_from_other_scopes_do_not_enter_the_snapshot_format() {
    let root: PathBuf = fixture("shadow-config-snapshot-scopes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let global: PathBuf = root.join("global");
    std::fs::write(global.as_path(), "[extensions]\n\tfromglobal = yes\n").expect("global config");
    let mut overlay = crate::test_support::isolated_overlay();
    overlay.retain(|(name, _value)| { return name != "GIT_CONFIG_GLOBAL"; });
    overlay.push((std::ffi::OsString::from("GIT_CONFIG_GLOBAL"), global.into_os_string()));
    overlay.push((std::ffi::OsString::from("GIT_CONFIG_COUNT"), std::ffi::OsString::from("1")));
    overlay.push((std::ffi::OsString::from("GIT_CONFIG_KEY_0"), std::ffi::OsString::from("extensions.fromenv")));
    overlay.push((std::ffi::OsString::from("GIT_CONFIG_VALUE_0"), std::ffi::OsString::from("1")));
    let context: GitContext = GitContext { overlay, ..git_context() };
    let snapshot: ConfigSnapshot =
        crate::config_snapshot::read_config_snapshot(&context, repo.as_path()).expect("snapshot");
    assert!(snapshot.entries().iter().any(|entry| { return entry.key == "extensions.fromglobal"; }));
    let from_snapshot: RepositoryFormat = repository_format_from_snapshot(&snapshot).expect("no includes");
    let from_file: RepositoryFormat = read_repository_format(&context, common.as_path()).expect("format");
    assert_eq!(from_snapshot, from_file);
    assert!(from_snapshot.extensions.iter().all(|(name, _value)| { return !name.starts_with("from"); }));
    remove(root.as_path());
}

/// An include of any kind, or a read made with global options, leaves the format to Git.
#[test]
fn includes_and_global_options_leave_the_format_to_git() {
    let root: PathBuf = fixture("shadow-config-snapshot-includes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    std::fs::write(common.join("extra"), "[extensions]\n\tfromextra = 1\n").expect("extra");
    let context: GitContext = git_context();
    for text in [
        "[include]\n\tpath = extra\n",
        "[includeIf \"gitdir:/\"]\n\tpath = extra\n",
        "[includeif \"gitdir:/nowhere/\"]\n\tpath = extra\n",
    ] {
        std::fs::write(common.join("config"), format!("[core]\n\trepositoryformatversion = 0\n{text}")).expect("config");
        let snapshot: ConfigSnapshot =
            crate::config_snapshot::read_config_snapshot(&context, repo.as_path()).expect("snapshot");
        assert!(repository_format_from_snapshot(&snapshot).is_none(), "{text}");
    }
    std::fs::write(common.join("config"), "[core]\n\trepositoryformatversion = 0\n").expect("config");
    let optioned: GitContext = GitContext {
        global_prefix: vec![std::ffi::OsString::from("-c"), std::ffi::OsString::from("x.y=z")],
        ..git_context()
    };
    let snapshot: ConfigSnapshot =
        crate::config_snapshot::read_config_snapshot(&optioned, repo.as_path()).expect("snapshot");
    assert!(repository_format_from_snapshot(&snapshot).is_none());
    remove(root.as_path());
}
