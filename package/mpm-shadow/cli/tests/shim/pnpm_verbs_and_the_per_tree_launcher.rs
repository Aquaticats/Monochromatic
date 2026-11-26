//! What:
//! The part of the shim tests that watches which program a word typed to `mpm` reaches:
//! the per-tree launcher of the running shadow is never taken for the real mpm (issue 6259),
//! every pnpm verb and alias the shadow declares runs the shadow's own command,
//! `mpm i -g` is the toolchain's global install,
//! a word only the real mpm has still reaches it with its arguments and status,
//! and a command line that starts with one of mpm's own options stays mpm's.
//! Why:
//! The owner wants pnpm's verbs on `mpm` itself
//! ("I want pnpm style verbs.
//! We talked about this!"),
//! and the recursion of 2026-11-03 showed how much depends on the shadow never starting a shadow:
//! each test proves the real mpm never ran by a marker file, not by the shape of the output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim/pnpm_verbs_and_the_per_tree_launcher.test.ts
//! test('every pnpm verb runs the shadow', () => { for (const word of declaredVerbs()) { ... } });
//! ```

use std::fs;
use std::path::Path;
use std::process::{
    Command,
    Output,
};

use mpm_shadow::cli::declared_verbs;

use crate::{
    fixture_root,
    start_shadow,
    start_shadow_with,
    write_fake_mpm,
};

/// The failure of 2026-11-03 (issue 6259):
/// the link `install` makes resolves to a per-tree launcher,
/// which starts the real shadow binary under the name `mpm`;
/// that shadow must not take the launcher for the real mpm and start it again.
#[test]
fn the_per_tree_launcher_of_the_running_shadow_is_not_taken_for_the_real_mpm() {
    use mpm_shadow::toolchain::launcher::{
        LaunchRecord,
        record_path,
        render_record,
    };
    use mpm_shadow::toolchain::scope::ScopeKind;
    let root = fixture_root();
    let runs: std::path::PathBuf = root.path().join("runs.log");
    write_fake_mpm(
        &root.path().join("real"),
        &format!("#!/bin/sh\necho run >> '{}'\nexit 5\n", runs.display()),
    );
    let binary: &str = env!("CARGO_BIN_EXE_mpm-shadow");
    let installed: std::path::PathBuf = root.path().join("toolchain/mpm-shadow/bin");
    fs::create_dir_all(&installed).expect("make the install directory");
    fs::copy(binary, installed.join("mpm-shadow")).expect("install the real shadow binary");
    let launchers: std::path::PathBuf = root.path().join("launchers");
    fs::create_dir_all(&launchers).expect("make the launcher directory");
    // A copy, not a hard link: the build directory and the fixture can be on different file systems.
    fs::copy(binary, launchers.join("mpm-shadow")).expect("make the launcher");
    let record = LaunchRecord {
        tool_id: "mpm-shadow".to_string(),
        lock_file: root.path().join("toolchain.lock.json"),
        real: installed.join("mpm-shadow"),
        scope: ScopeKind::Repo,
    };
    fs::write(
        record_path(&launchers.join("mpm-shadow")),
        render_record(&record),
    )
    .expect("write the launch record");
    let links: std::path::PathBuf = root.path().join("links");
    fs::create_dir_all(&links).expect("make the link directory");
    std::os::unix::fs::symlink(launchers.join("mpm-shadow"), links.join("mpm"))
        .expect("link the launcher as mpm");
    let path_variable: String = format!(
        "{}:{}:/usr/bin:/bin",
        links.display(),
        root.path().join("real").display()
    );

    let output: Output = Command::new(links.join("mpm"))
        .arg("installed")
        .env("PATH", path_variable)
        .env("CARGO_HOME", root.path().join("cargo"))
        .env("HOME", root.path().join("home"))
        .env("XDG_DATA_HOME", root.path().join("data"))
        .output()
        .expect("run the launcher");

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(5),
        "the real mpm's own status: {stderr}"
    );
    assert_eq!(
        fs::read_to_string(&runs)
            .expect("the real mpm ran")
            .lines()
            .count(),
        1,
        "{stderr}"
    );
}

/// What:
/// A fake real mpm that leaves a marker file when it runs,
/// so a test can tell whether the shadow started it.
/// Why:
/// The pnpm-verb tests must prove the real mpm never ran,
/// not only that the output looks right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeMarkingMpm(root: string): string // returns the marker path
/// ```
fn write_marking_mpm(root: &Path) -> std::path::PathBuf {
    let marker: std::path::PathBuf = root.join("real-mpm-ran");
    write_fake_mpm(
        &root.join("real"),
        &format!("#!/bin/sh\necho \"$@\" >> '{}'\nexit 7\n", marker.display()),
    );
    return marker;
}

/// What:
/// Every verb and alias the shadow declares,
/// typed to `mpm` as its first word,
/// runs the shadow's own command:
///           `mpm <word> --help` prints that command's help and exits 0,
/// and the real mpm never starts.
/// Why:
/// The owner wants pnpm's verbs on `mpm` itself:
/// "I want pnpm style verbs.
/// We talked about this!"
/// (2026-11-03,
///           after `mpm i -g` reached mpm 8.2.0 and failed with "No such command 'i'").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const verb of declaredVerbs()) for (const word of [verb.name, ...verb.aliases]) {
///   const result = startShadow(root, [word, '--help']);
///   expect(result.status).toBe(0); expect(realMpmRan).toBe(false);
/// }
/// ```
#[test]
fn every_pnpm_verb_and_alias_typed_to_mpm_runs_the_shadows_own_command_and_never_the_real_mpm() {
    let root = fixture_root();
    let marker: std::path::PathBuf = write_marking_mpm(root.path());
    for verb in declared_verbs() {
        // The verb's name first, then each alias the table gives it.
        let mut words: Vec<&str> = vec![verb.name];
        words.extend(verb.aliases.iter().copied());
        for word in words {
            let output: Output = start_shadow(root.path(), &[word, "--help"]);
            let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
            assert_eq!(
                output.status.code(),
                Some(0),
                "mpm {word} --help: {stdout}{stderr}"
            );
            assert!(
                stdout.contains(&format!("mpm {}", verb.name)),
                "mpm {word} --help prints the {} help: {stdout}",
                verb.name
            );
            assert!(!marker.exists(), "mpm {word} started the real mpm");
        }
    }
}

/// What:
/// The command of issue 6259,
/// `mpm i -g`,
/// is the toolchain's global install:
/// with no machine-global plan in the fixture
///           it is refused by the shadow,
/// and the real mpm is never started.
/// Why:
/// It is the exact command the owner typed;
/// the config home points into the fixture so the machine's own global plan is never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = startShadow(root, ['i', '-g'], { XDG_CONFIG_HOME: join(root, 'config') });
/// expect(realMpmRan).toBe(false); expect(result.stderr).not.toContain("No such command");
/// ```
#[test]
fn mpm_i_g_is_the_toolchains_global_install_and_never_reaches_the_real_mpm() {
    let root = fixture_root();
    let marker: std::path::PathBuf = write_marking_mpm(root.path());
    let config: String = root.path().join("config").display().to_string();

    let output: Output =
        start_shadow_with(root.path(), &["i", "-g"], &[("XDG_CONFIG_HOME", &config)]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(!marker.exists(), "mpm i -g started the real mpm: {stderr}");
    assert!(!stderr.contains("No such command"), "{stderr}");
    assert!(
        stderr.contains("toolchain"),
        "the shadow's own install answered: {stderr}"
    );
}

/// What:
/// A word only the real mpm has (`installed`) still reaches it,
/// with every argument untouched and its exit status passed through.
/// Why:
/// Only pnpm's verbs moved to the shadow;
/// mpm's own commands keep working through `mpm`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = startShadow(root, ['installed', '--manager', 'pip']);
/// expect(readMarker()).toBe('installed --manager pip\n'); expect(result.status).toBe(7);
/// ```
#[test]
fn a_word_only_the_real_mpm_knows_still_reaches_it_with_its_arguments_and_status() {
    let root = fixture_root();
    let marker: std::path::PathBuf = write_marking_mpm(root.path());

    let output: Output = start_shadow(root.path(), &["installed", "--manager", "pip"]);

    assert_eq!(
        output.status.code(),
        Some(7),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&marker).expect("the real mpm ran"),
        "installed --manager pip\n"
    );
}

/// What:
/// A command line that starts with one of mpm's own options stays mpm's,
/// even when a pnpm verb follows:
///           `mpm --verbosity DEBUG outdated` runs the real mpm's `outdated`.
/// Why:
/// pnpm verbs are typed first (`mpm i -g`);
/// mpm's top-level options belong to the real mpm,
/// so such a line is the real mpm's command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startShadow(root, ['--verbosity', 'DEBUG', 'outdated']); expect(realMpmRan).toBe(true);
/// ```
#[test]
fn a_command_line_that_starts_with_an_mpm_option_stays_the_real_mpms_even_before_a_pnpm_verb() {
    let root = fixture_root();
    let marker: std::path::PathBuf = write_marking_mpm(root.path());

    let output: Output = start_shadow(root.path(), &["--verbosity", "DEBUG", "outdated"]);

    assert_eq!(
        output.status.code(),
        Some(7),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&marker).expect("the real mpm ran"),
        "--verbosity DEBUG outdated\n"
    );
}
