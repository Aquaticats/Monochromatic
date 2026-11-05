//! What:
//!  Disposable-directory controls for the settings file's two spellings.
//! Why:
//!  The owner renamed `cli-git.config.jsonc` to `monochromatic-git-policy-cli.config.jsonc` and decided that a repository
//!  with only the old file is read with one notice that names the new name, that a repository with both reads the new one
//!  and reports the old one as ignored, and that the wrapper never renames a committed file by itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeFile(join(root, 'cli-git.config.jsonc'), '{}'); expect((await load(root)).earlierNameNotice).toContain('monochromatic-git-policy-cli.config.jsonc');
//! ```

/// The loader under test,
///  both spellings and the settings types the files produce.
use super::{CONFIG_FILE_NAME, EARLIER_CONFIG_FILE_NAME, LoadedConfig, load_repository_config, settings_file_of};
use crate::config_schema::CliGitConfig;
use crate::policy_registry::{PolicyId, Severity};
use crate::wrapper_names_old_and_new::SETTINGS_FILE;
use std::path::{Path, PathBuf};

/// Create one empty fixture directory owned by this test process and test name.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-config-earlier", name);
}

/// A settings document that sets one policy to the given severity word.
fn document(severity: &str) -> String {
    return format!("{{ \"policies\": {{ \"final-newline\": \"{severity}\" }} }}");
}

/// The severity the loaded settings give the final-newline policy.
fn final_newline(loaded: &LoadedConfig) -> Severity {
    return loaded.config.policies.setting(PolicyId::FinalNewline).severity;
}

/// The two spellings are the pair the program keeps.
#[test]
fn the_loader_reads_the_pair_of_settings_names() {
    assert_eq!(CONFIG_FILE_NAME, SETTINGS_FILE.current);
    assert_eq!(EARLIER_CONFIG_FILE_NAME, SETTINGS_FILE.earlier);
}

/// Only the earlier file:
///  it is read,
///  and one line names the new file.
#[test]
fn the_earlier_file_alone_is_read_with_one_notice_naming_the_new_name() {
    let root: PathBuf = fixture("earlier-only");
    std::fs::write(root.join(EARLIER_CONFIG_FILE_NAME), document("error")).expect("write the earlier file");
    let loaded: LoadedConfig = load_repository_config(root.as_path()).expect("the earlier file loads");
    assert_eq!(final_newline(&loaded), Severity::Error);
    assert_eq!(loaded.source, Some(root.join(EARLIER_CONFIG_FILE_NAME)));
    let notice: String = loaded.earlier_name_notice.expect("one notice");
    assert!(notice.contains(CONFIG_FILE_NAME) && notice.contains(EARLIER_CONFIG_FILE_NAME), "{notice}");
    assert_eq!(notice.matches('\n').count(), 1, "one line: {notice:?}");
    assert!(notice.ends_with('\n'));
    assert!(root.join(EARLIER_CONFIG_FILE_NAME).is_file(), "the wrapper never renames a committed file");
    assert!(!root.join(CONFIG_FILE_NAME).exists(), "and never creates the new one");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Only the new file:
///  it is read and nothing is said.
#[test]
fn the_new_file_alone_is_read_without_a_notice() {
    let root: PathBuf = fixture("new-only");
    std::fs::write(root.join(CONFIG_FILE_NAME), document("error")).expect("write the new file");
    let loaded: LoadedConfig = load_repository_config(root.as_path()).expect("the new file loads");
    assert_eq!(final_newline(&loaded), Severity::Error);
    assert_eq!(loaded.source, Some(root.join(CONFIG_FILE_NAME)));
    assert_eq!(loaded.earlier_name_notice, None);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Both files:
///  the new one is read,
///  the earlier one is ignored whatever it says,
///  and the one line names the earlier file as ignored.
#[test]
fn both_files_read_the_new_one_and_report_the_earlier_as_ignored() {
    let root: PathBuf = fixture("both");
    std::fs::write(root.join(CONFIG_FILE_NAME), document("warn")).expect("write the new file");
    std::fs::write(root.join(EARLIER_CONFIG_FILE_NAME), document("error")).expect("write the earlier file");
    let loaded: LoadedConfig = load_repository_config(root.as_path()).expect("the new file loads");
    assert_eq!(final_newline(&loaded), Severity::Warn, "the earlier file's settings are not read");
    assert_eq!(loaded.source, Some(root.join(CONFIG_FILE_NAME)));
    let notice: String = loaded.earlier_name_notice.expect("one notice");
    assert!(notice.contains(EARLIER_CONFIG_FILE_NAME) && notice.contains("ignored"), "{notice}");
    assert_eq!(notice.matches('\n').count(), 1, "one line: {notice:?}");
    assert!(root.join(EARLIER_CONFIG_FILE_NAME).is_file(), "the earlier file is left as it is");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// An earlier file that is invalid is an error when it is the one read,
///  and not when the new file shadows it.
#[test]
fn an_invalid_earlier_file_fails_only_when_it_is_the_one_read() {
    let root: PathBuf = fixture("invalid-earlier");
    std::fs::write(root.join(EARLIER_CONFIG_FILE_NAME), "{ \"unknown\": 1 }").expect("write the earlier file");
    assert!(load_repository_config(root.as_path()).is_err());
    std::fs::write(root.join(CONFIG_FILE_NAME), "{}").expect("write the new file");
    assert!(load_repository_config(root.as_path()).is_ok(), "the new file shadows the invalid one");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Neither file:
///  the defaults,
///  no source,
///  and nothing said.
#[test]
fn neither_file_gives_the_defaults_and_no_notice() {
    let root: PathBuf = fixture("neither");
    let loaded: LoadedConfig = load_repository_config(root.as_path()).expect("defaults");
    assert_eq!(loaded.config, CliGitConfig::defaults());
    assert_eq!(loaded.source, None);
    assert_eq!(loaded.earlier_name_notice, None);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A TypeScript-era file beside the earlier JSONC file is still reported as ignored,
///  and the earlier JSONC file is still read.
#[test]
fn a_typescript_era_file_beside_the_earlier_file_is_still_reported() {
    let root: PathBuf = fixture("legacy-beside-earlier");
    std::fs::write(root.join(EARLIER_CONFIG_FILE_NAME), document("error")).expect("write the earlier file");
    std::fs::write(root.join("cli-git.config.mjs"), "export default {};").expect("write the module");
    let loaded: LoadedConfig = load_repository_config(root.as_path()).expect("loads");
    assert_eq!(final_newline(&loaded), Severity::Error);
    assert_eq!(loaded.ignored_legacy, vec![root.join("cli-git.config.mjs")]);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A TypeScript-era file with neither JSONC file is still a migration error,
///  and it names the new file as the place for the translation.
#[test]
fn a_typescript_era_file_alone_names_the_new_file_in_its_error() {
    let root: PathBuf = fixture("legacy-alone");
    std::fs::write(root.join("cli-git.config.ts"), "export default {};").expect("write the module");
    let error: String = load_repository_config(root.as_path()).expect_err("migration is required").message;
    assert!(error.contains(CONFIG_FILE_NAME), "{error}");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A symbolic link under the earlier name is refused like one under the new name.
#[test]
fn a_symbolic_link_under_the_earlier_name_is_refused() {
    let root: PathBuf = fixture("earlier-link");
    let elsewhere: PathBuf = root.join("elsewhere.jsonc");
    std::fs::write(elsewhere.as_path(), "{}").expect("write the target");
    std::os::unix::fs::symlink(elsewhere.as_path(), root.join(EARLIER_CONFIG_FILE_NAME)).expect("link");
    let error: String = load_repository_config(root.as_path()).expect_err("a link is refused").message;
    assert!(error.contains("symbolic link"), "{error}");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Spaces,
///  quotes and a newline in the repository path do not change what is read,
///  and the notice stays one line because it names files and never the path.
#[test]
fn an_awkward_repository_path_does_not_change_the_notice_or_what_is_read() {
    let outer: PathBuf = fixture("awkward");
    let root: PathBuf = outer.join("a repo's \"top\"\nlevel");
    std::fs::create_dir_all(root.as_path()).expect("create the repository");
    std::fs::write(root.join(EARLIER_CONFIG_FILE_NAME), document("error")).expect("write the earlier file");
    let loaded: LoadedConfig = load_repository_config(Path::new(root.as_path())).expect("loads");
    assert_eq!(final_newline(&loaded), Severity::Error);
    let notice: String = loaded.earlier_name_notice.expect("one notice");
    assert_eq!(notice.matches('\n').count(), 1, "one line: {notice:?}");
    assert!(!notice.contains("level"), "the notice names files, not the path: {notice:?}");
    std::fs::remove_dir_all(outer.as_path()).expect("remove only the fixture");
}

/// The file a repository's settings come from is found the way the loader finds it:
///  the new name when it exists,
///  else the earlier name when only that exists,
///  else the new name.
///  A control that must read a repository's own file asks this and so works before and after the file is renamed.
#[test]
fn the_settings_file_of_a_repository_is_found_current_name_first_then_the_earlier() {
    let root: PathBuf = fixture("settings-file-of");
    assert_eq!(settings_file_of(root.as_path()), root.join(CONFIG_FILE_NAME), "neither");
    std::fs::write(root.join(EARLIER_CONFIG_FILE_NAME), "{}").expect("write the earlier file");
    assert_eq!(settings_file_of(root.as_path()), root.join(EARLIER_CONFIG_FILE_NAME), "earlier only");
    std::fs::write(root.join(CONFIG_FILE_NAME), "{}").expect("write the new file");
    assert_eq!(settings_file_of(root.as_path()), root.join(CONFIG_FILE_NAME), "both");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}
