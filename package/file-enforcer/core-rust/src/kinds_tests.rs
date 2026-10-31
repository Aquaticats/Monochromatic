//! What:
//!  Tests for the built-in rule kinds, each run through the real `run` entry point on a scratch worktree.
//! Why:
//!  A kind is correct when `write` converges, a second `check` is clean, and unowned files survive.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('rule kinds', () => { it('concat joins parts', () => { ... }); });
//! ```

use crate::run::RunMode;
use crate::test_support::{count_code, read_text, run_config, scratch_dir, write_files};
use std::fs;
use std::os::unix::fs::PermissionsExt;

/// Check mode with no expected directory.
fn check() -> RunMode {
    return RunMode::Check { expected_dir: None };
}

#[test]
fn concat_joins_parts_sets_the_mode_and_converges() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("pre.md", "preamble\n\n"), ("AGENTS.md", "body\n")]);
    let config = r#"{"rules": [{"kind": "concat", "id": "claude", "output": "CLAUDE.md", "mode": "0600",
        "parts": [{"file": "pre.md"}, {"file": "AGENTS.md"}]}]}"#;
    let (code, _) = run_config(dir.path(), config, check());
    assert_eq!(code, 1, "a missing output is drift");
    let (code, _) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0);
    assert_eq!(read_text(dir.path(), "CLAUDE.md"), "preamble\n\nbody\n");
    let mode = fs::metadata(dir.path().join("CLAUDE.md")).expect("exists").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let (code, events) = run_config(dir.path(), config, check());
    assert_eq!(code, 0, "{events:?}");
}

#[test]
fn mode_only_repairs_an_existing_file_and_never_creates_one() {
    let dir = scratch_dir();
    let config = r#"{"rules": [{"kind": "mode-only", "id": "m", "path": "secret.txt", "mode": "0600"}]}"#;
    let (code, _) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0);
    assert!(!dir.path().join("secret.txt").exists());
    write_files(dir.path(), &[("secret.txt", "x")]);
    fs::set_permissions(dir.path().join("secret.txt"), fs::Permissions::from_mode(0o644)).expect("chmod");
    let (code, events) = run_config(dir.path(), config, check());
    assert_eq!(code, 1);
    assert_eq!(count_code(&events, "drift-mode"), 1, "{events:?}");
    run_config(dir.path(), config, RunMode::Write);
    let mode = fs::metadata(dir.path().join("secret.txt")).expect("exists").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn package_licenses_copies_required_texts_and_prunes_only_owned_ones() {
    let dir = scratch_dir();
    write_files(
        dir.path(),
        &[
            ("LICENSES/GPL-3.0-or-later.txt", "gpl"),
            ("LICENSES/LGPL-3.0-or-later.txt", "lgpl"),
            ("LICENSES/CC-BY-SA-4.0.txt", "cc"),
            ("package/a/b/package.json", r#"{"license": "LGPL-3.0-or-later"}"#),
            ("package/c/d/Cargo.toml", "[package]\nlicense = \"MIT\"\n"),
            ("package/c/d/LICENSES/GPL-3.0-or-later.txt", "old"),
            ("package/c/d/LICENSES/MIT.txt", "hand kept"),
            ("package/c/d/node_modules/e/package.json", r#"{"license": "LGPL-3.0-or-later"}"#),
        ],
    );
    let config = r#"{"rules": [{"kind": "package-licenses", "id": "lic", "directory": "LICENSES",
        "manifests": ["package/*/*/package.json", "package/*/*/Cargo.toml"],
        "texts": {"CC-BY-SA-4.0": "LICENSES/CC-BY-SA-4.0.txt", "GPL-3.0-or-later": "LICENSES/GPL-3.0-or-later.txt",
                  "LGPL-3.0-or-later": "LICENSES/LGPL-3.0-or-later.txt"},
        "requires": [{"contains": "LGPL-3.0-or-later", "texts": ["GPL-3.0-or-later", "LGPL-3.0-or-later"]},
                     {"contains": "CC-BY-SA-4.0", "texts": ["CC-BY-SA-4.0"]}]}]}"#;
    let (code, events) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0, "{events:?}");
    assert_eq!(read_text(dir.path(), "package/a/b/LICENSES/GPL-3.0-or-later.txt"), "gpl");
    assert_eq!(read_text(dir.path(), "package/a/b/LICENSES/LGPL-3.0-or-later.txt"), "lgpl");
    assert!(!dir.path().join("package/c/d/LICENSES/GPL-3.0-or-later.txt").exists(), "owned stale text is pruned");
    assert_eq!(read_text(dir.path(), "package/c/d/LICENSES/MIT.txt"), "hand kept");
    assert!(!dir.path().join("package/c/d/node_modules/e/LICENSES").exists(), "the glob is exactly two levels deep");
    let (code, _) = run_config(dir.path(), config, check());
    assert_eq!(code, 0);
}

#[test]
fn mirror_makes_the_target_equal_and_leaves_siblings_alone() {
    let dir = scratch_dir();
    write_files(
        dir.path(),
        &[
            ("src/one/SKILL.md", "one"),
            ("src/two.md", "two"),
            (".claude/skills/one/SKILL.md", "stale"),
            (".claude/skills/gone/old.md", "old"),
            (".claude/skills/.agents-mirror-manifest.json", "{}"),
            (".claude/other/keep.txt", "keep"),
        ],
    );
    let config = r#"{"rules": [{"kind": "mirror", "id": "m", "from": "src", "to": ".claude/skills"}]}"#;
    let (code, events) = run_config(dir.path(), config, check());
    assert_eq!(code, 1);
    assert!(count_code(&events, "drift-extra-directory") >= 1, "{events:?}");
    let (code, events) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0, "{events:?}");
    assert_eq!(read_text(dir.path(), ".claude/skills/one/SKILL.md"), "one");
    assert_eq!(read_text(dir.path(), ".claude/skills/two.md"), "two");
    assert!(!dir.path().join(".claude/skills/gone").exists());
    assert!(!dir.path().join(".claude/skills/.agents-mirror-manifest.json").exists());
    assert_eq!(read_text(dir.path(), ".claude/other/keep.txt"), "keep", "a sibling of the target is never touched");
    let (code, events) = run_config(dir.path(), config, check());
    assert_eq!(code, 0, "{events:?}");
}

#[test]
fn mirror_rejects_a_symlink_in_the_source_and_removes_one_in_the_target() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("src/a.md", "a"), ("outside.txt", "outside")]);
    let config = r#"{"rules": [{"kind": "mirror", "id": "m", "from": "src", "to": "dst"}]}"#;
    std::os::unix::fs::symlink("../outside.txt", dir.path().join("src/link.md")).expect("symlink");
    let (code, events) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 1);
    assert_eq!(count_code(&events, "symlink-rejected"), 1, "{events:?}");
    assert!(!dir.path().join("dst").exists(), "a failing run writes nothing");
    fs::remove_file(dir.path().join("src/link.md")).expect("remove");
    write_files(dir.path(), &[("dst/a.md", "a")]);
    std::os::unix::fs::symlink("../outside.txt", dir.path().join("dst/planted.md")).expect("symlink");
    let (code, _) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0);
    assert!(fs::symlink_metadata(dir.path().join("dst/planted.md")).is_err(), "the link is removed as a link");
    assert_eq!(read_text(dir.path(), "outside.txt"), "outside", "the link target is never touched");
}

#[test]
fn toml_keys_sets_guarded_keys_appends_blocks_and_keeps_comments() {
    let dir = scratch_dir();
    let manifest = "# my crate\n[package]\nname = \"a\"\nedition = \"2021\" # old\nlicense = \"MIT\"\n\n[lints.clippy]\nimplicit_return = \"warn\"\n";
    write_files(dir.path(), &[("package/x/y/Cargo.toml", manifest), ("package/x/z/Cargo.toml", "[package]\nname = \"z\"\n"),
        ("block.toml", "[workspace]\n")]);
    let config = r#"{"rules": [{"kind": "toml-keys", "id": "cargo", "manifests": ["package/*/*/Cargo.toml"],
        "edits": [
          {"guard": ["package"], "path": ["package", "edition"], "value": "2024"},
          {"guard": ["package", "license"], "path": ["package", "license"], "value": "LGPL-3.0-or-later"},
          {"guard": ["package", "publish"], "path": ["package", "publish"], "value": false},
          {"guard": ["lints", "clippy"], "path": ["lints", "clippy", "implicit_return"], "value": "deny"},
          {"guard": ["package"], "path": ["package", "homepage"], "value": {"concat": ["https://example/tree/", {"var": "dir"}]}}
        ],
        "blocks": [{"absentPath": ["workspace"], "text": {"file": "block.toml"}}]}]}"#;
    let (code, events) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0, "{events:?}");
    let first = read_text(dir.path(), "package/x/y/Cargo.toml");
    assert!(first.starts_with("# my crate\n"), "{first}");
    assert!(first.contains("edition = \"2024\" # old"), "the trailing comment survives: {first}");
    assert!(first.contains("license = \"LGPL-3.0-or-later\""), "{first}");
    assert!(first.contains("implicit_return = \"deny\""), "{first}");
    assert!(first.contains("homepage = \"https://example/tree/package/x/y\""), "{first}");
    assert!(!first.contains("publish"), "a key whose guard is absent is not added: {first}");
    assert!(first.ends_with("\n[workspace]\n"), "{first}");
    assert!(read_text(dir.path(), "package/x/z/Cargo.toml").contains("edition = \"2024\""));
    let (code, events) = run_config(dir.path(), config, check());
    assert_eq!(code, 0, "{events:?}");
}

#[test]
fn mise_path_list_lists_sorted_non_empty_directories_only() {
    let dir = scratch_dir();
    write_files(
        dir.path(),
        &[("package/b/c/node_modules/.bin/tool", "x"), ("package/a/c/node_modules/.bin/tool", "x")],
    );
    fs::create_dir_all(dir.path().join("package/z/c/node_modules/.bin")).expect("empty bin");
    let config = r##"{"rules": [{"kind": "mise-path-list", "id": "bin", "output": ".config/mise/conf.d/10-bin.local.toml",
        "directories": ["package/*/*/node_modules/.bin"], "header": "# generated\n"}]}"##;
    let (code, events) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0, "{events:?}");
    let text = read_text(dir.path(), ".config/mise/conf.d/10-bin.local.toml");
    assert_eq!(
        text,
        "# generated\n[env]\n_.path = [\n  \"package/a/c/node_modules/.bin\",\n  \"package/b/c/node_modules/.bin\",\n]\n"
    );
}

#[test]
fn a_kind_whose_library_is_not_linked_warns_and_is_not_drift() {
    let dir = scratch_dir();
    let config = r#"{"rules": [{"kind": "browserslist", "id": "bl", "source": ".browserslistrc", "output": "out.json"}]}"#;
    let (code, events) = run_config(dir.path(), config, check());
    assert_eq!(code, 0, "{events:?}");
    assert_eq!(count_code(&events, "kind-not-wired"), 1);
}
