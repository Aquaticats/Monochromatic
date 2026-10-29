//! Real executable help and usage must finish before any project or native display startup.
#![cfg(feature = "gui")]

/// Subprocess outputs verify exit status and streams at the actual CLI boundary;
///  times age the
/// seeded cache folders.
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{Duration, SystemTime},
};

/// Isolate native display endpoints and private state even if startup ordering regresses.
fn invoke(root: &Path, args: &[&OsStr]) -> Output {
    // What: Command builds a child process; env_remove removes inherited host display fallbacks.
    // Why: A CLI regression must not open a window or use the human's display while this test runs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const child = spawn(executable, args, { cwd: fixture, env: isolatedEnvironment });
    // ```
    let mut command = Command::new(env!("CARGO_BIN_EXE_monochromatic-ide"));
    command.current_dir(root).args(args);
    command.env("SLINT_BACKEND", "winit");
    command.env("WAYLAND_DISPLAY", root.join("absent-wayland.socket"));
    command.env("XDG_RUNTIME_DIR", root);
    command.env("XDG_CONFIG_HOME", root.join("config"));
    command.env("XDG_CACHE_HOME", root.join("cache"));
    command.env("XDG_DATA_HOME", root.join("data"));
    // A disposable home folder: a start without a project argument opens it, never the real one.
    command.env("HOME", root.join("home"));
    command.env_remove("DISPLAY");
    command.env_remove("WAYLAND_SOCKET");
    return command.output().expect("run native CLI");
}

/// Real help/version print only to stdout,
///  exit zero,
///  and do not create private state or read a missing root.
#[test]
fn executable_help_and_version_exit_without_startup() {
    let fixture = tempfile::tempdir().expect("disposable CLI environment");
    let missing = fixture.path().join("missing-project");
    for flag in ["--help", "--version"] {
        let output = invoke(fixture.path(), &[missing.as_os_str(), OsStr::new(flag)]);
        assert!(output.status.success(), "{flag}: {:?}", output.stderr);
        assert!(!output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).expect("UTF-8 CLI output");
        assert!(text.contains("monochromatic-ide"));
        assert!(!text.contains("opened read-only project"));
    }
    assert_eq!(
        fs::read_dir(fixture.path())
            .expect("private directory")
            .count(),
        0
    );
}

/// Bad option grammar has status 2 and stderr,
///  not an attempted native backend connection.
#[test]
fn executable_usage_errors_exit_before_startup() {
    let fixture = tempfile::tempdir().expect("disposable CLI environment");
    for args in [
        vec![OsStr::new("--unknown")],
        vec![OsStr::new("one"), OsStr::new("two")],
    ] {
        let output = invoke(fixture.path(), &args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");
        assert!(text.contains("Usage:"));
        assert!(!text.contains("wayland"));
    }
    assert_eq!(
        fs::read_dir(fixture.path())
            .expect("private directory")
            .count(),
        0
    );
}

/// Positive filesystem controls prove valid argument grammar reaches root/source validation before GUI startup.
#[test]
fn invalid_project_and_non_regular_source_have_input_specific_errors() {
    let fixture = tempfile::tempdir().expect("disposable CLI environment");
    let project = fixture.path().join("project");
    let missing = fixture.path().join("missing");
    fs::create_dir(&project).expect("project fixture");
    let absent = invoke(fixture.path(), &[missing.as_os_str()]);
    assert!(!absent.status.success());
    let absent_text = String::from_utf8(absent.stderr).expect("UTF-8 missing-root diagnostic");
    assert!(absent_text.contains("Cannot open project directory"));
    assert!(absent_text.contains(&missing.display().to_string()));
    let directory = invoke(
        fixture.path(),
        &[project.as_os_str(), OsStr::new("--file"), OsStr::new(".")],
    );
    assert!(!directory.status.success());
    let directory_text = String::from_utf8(directory.stderr).expect("UTF-8 source diagnostic");
    assert!(directory_text.contains("not a regular file"));
    assert!(directory_text.contains(&project.display().to_string()));
    assert_eq!(
        fs::read_dir(fixture.path())
            .expect("private directory")
            .count(),
        1
    );
}

/// Without a project argument the disposable home folder is opened:
///  startup gets past the project
/// checks to the display connection,
///  which fails in this test on purpose.
///  Without any home folder
/// the grammar reports a usage error before anything starts.
#[test]
fn missing_project_opens_the_home_folder_or_reports_its_absence() {
    let fixture = tempfile::tempdir().expect("disposable CLI environment");
    fs::create_dir(fixture.path().join("home")).expect("disposable home folder");
    let started = invoke(fixture.path(), &[]);
    let started_text = String::from_utf8_lossy(&started.stderr).into_owned();
    assert_ne!(started.status.code(), Some(2), "{started_text}");
    assert!(!started_text.contains("Usage:"), "{started_text}");
    assert!(
        !started_text.contains("Cannot open project directory"),
        "{started_text}"
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_monochromatic-ide"));
    command.current_dir(fixture.path()).env_remove("HOME");
    command.env(
        "WAYLAND_DISPLAY",
        fixture.path().join("absent-wayland.socket"),
    );
    command.env_remove("DISPLAY");
    let refused = command.output().expect("run native CLI without HOME");
    assert_eq!(refused.status.code(), Some(2));
    let refused_text = String::from_utf8(refused.stderr).expect("UTF-8 diagnostic");
    assert!(
        refused_text.contains("No PROJECT was given"),
        "{refused_text}"
    );
}

/// Every file below `directory`,
///  as (path relative to it with `/` separators,
///  full path),
///  sorted.
fn files_below(directory: &Path) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let mut pending = vec![directory.to_path_buf()];
    while let Some(folder) = pending.pop() {
        for listed in fs::read_dir(&folder).expect("readable folder") {
            let path = listed.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let relative = path
                .strip_prefix(directory)
                .expect("below the folder")
                .to_string_lossy()
                .into_owned();
            found.push((relative, path));
        }
    }
    found.sort();
    return found;
}

/// The license and notice texts the executable must carry,
///  read from the files its build embedded:
/// every grammar notice,
///  Helix's license and the license files among its queries,
///  the application's
/// license texts,
///  and the two font notices.
fn expected_notices() -> Vec<(String, PathBuf)> {
    let runtime = Path::new(env!("CARGO_BIN_EXE_monochromatic-ide"))
        .parent()
        .expect("target profile folder")
        .join("runtime");
    let package = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut expected: Vec<(String, PathBuf)> = Vec::new();
    for (relative, path) in files_below(&runtime.join("licenses")) {
        expected.push((format!("runtime/licenses/{relative}"), path));
    }
    expected.push((
        "runtime/Helix-LICENSE".to_string(),
        runtime.join("Helix-LICENSE"),
    ));
    for (relative, path) in files_below(&runtime.join("queries")) {
        let name = relative
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        if name.contains("LICENSE")
            || name.contains("LICENCE")
            || name.starts_with("COPYING")
            || name.starts_with("NOTICE")
        {
            expected.push((format!("runtime/queries/{relative}"), path));
        }
    }
    for (relative, path) in files_below(&package.join("LICENSES")) {
        expected.push((format!("LICENSES/{relative}"), path));
    }
    for notice in ["Inter-LICENSE.txt", "JetBrainsMono-OFL.txt"] {
        expected.push((
            format!("LICENSES/font/{notice}"),
            package.join("asset/font").join(notice),
        ));
    }
    return expected;
}

/// The Rust crate license list the executable embeds:
///  target/crate-licenses.json,
///  which the
/// `notices` task writes with cargo-about beside the profile folders.
fn crate_licenses() -> Vec<serde_json::Value> {
    let list = Path::new(env!("CARGO_BIN_EXE_monochromatic-ide"))
        .parent()
        .and_then(Path::parent)
        .expect("target folder")
        .join("crate-licenses.json");
    let parsed: serde_json::Value =
        serde_json::from_slice(&fs::read(&list).expect("crate license list of the build"))
            .expect("JSON crate license list");
    return parsed["licenses"]
        .as_array()
        .expect("licenses array")
        .clone();
}

/// `--licenses` prints every embedded license and notice text in full,
///  each under a heading that
/// names its component and its embedded path,
///  then every Rust crate license text under a heading
/// naming the license and the crates it covers,
///  and exits 0 without a display,
///  a project,
///  or a home
/// folder,
///  writing nothing.
#[test]
fn licenses_prints_every_embedded_text_without_a_display_or_home() {
    let fixture = tempfile::tempdir().expect("disposable CLI environment");
    let output = invoke(fixture.path(), &[OsStr::new("--licenses")]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout.clone()).expect("UTF-8 license texts");
    let expected = expected_notices();
    assert!(
        expected.len() > 30,
        "the build inputs hold the grammar notices: {expected:?}"
    );
    let crates = crate_licenses();
    assert!(
        crates.len() > 100,
        "the crate license list holds the crates' texts: {}",
        crates.len()
    );
    let introduction = format!(
        "Monochromatic IDE {} carries these {} license and notice texts, each in full below.\n",
        env!("CARGO_PKG_VERSION"),
        expected.len() + crates.len()
    );
    assert!(
        text.starts_with(&introduction),
        "{}",
        text.lines().next().unwrap_or_default()
    );
    assert_eq!(text.matches("\nEmbedded as ").count(), expected.len());
    let rule = "=".repeat(78);
    for (path, source) in &expected {
        let body = fs::read_to_string(source).expect("license text of the build inputs");
        let block = format!("\nEmbedded as {path}\n{rule}\n\n{body}");
        assert!(text.contains(&block), "{path} is not printed in full");
    }
    for heading in [
        "Monochromatic IDE: LGPL-3.0-or-later.txt",
        "Monochromatic IDE: GPL-3.0-or-later.txt",
        "Font compiled into the application: Inter-LICENSE.txt",
        "Font compiled into the application: JetBrainsMono-OFL.txt",
        "Helix (highlighting queries and the Helix crates compiled in): Helix-LICENSE",
        "Language grammar rust: LICENSE",
        "Language grammar slint: REUSE-headers.txt",
        "Helix highlighting queries for snakemake: LICENSE",
    ] {
        assert!(
            text.contains(&format!("\n{rule}\n{heading}\nEmbedded as ")),
            "missing heading {heading}"
        );
    }
    assert_eq!(text.matches("\nRust crates under ").count(), crates.len());
    assert!(!text.contains("Embedded as LICENSES/crates.json"));
    for license in &crates {
        let count = license["crates"].as_array().expect("crates").len();
        let noun = if count == 1 { "crate" } else { "crates" };
        let heading = format!(
            "\n{rule}\nRust crates under {} ({}): {count} {noun}\nUsed by: ",
            license["name"].as_str().expect("name"),
            license["id"].as_str().expect("id")
        );
        assert!(text.contains(&heading), "missing {heading}");
        let body = format!("{rule}\n\n{}", license["text"].as_str().expect("text"));
        assert!(
            text.contains(&body),
            "a text of {heading} is not printed in full"
        );
    }
    // Every crate of the list is named by name and version under some heading.
    let mut used_crates: Vec<&serde_json::Value> = Vec::new();
    for license in crates.iter() {
        for used in license["crates"].as_array().expect("crates").iter() {
            used_crates.push(used);
        }
    }
    for used in used_crates {
        let named = format!(
            "{} {}",
            used["name"].as_str().expect("crate name"),
            used["version"].as_str().expect("crate version")
        );
        assert!(text.contains(&named), "{named} is not named");
    }
    assert_eq!(
        fs::read_dir(fixture.path())
            .expect("private directory")
            .count(),
        0,
        "the listing wrote nothing"
    );
    let mut bare = Command::new(env!("CARGO_BIN_EXE_monochromatic-ide"));
    bare.current_dir(fixture.path()).arg("--licenses");
    bare.env_clear();
    let without_anything = bare.output().expect("run with an empty environment");
    assert!(without_anything.status.success());
    assert_eq!(without_anything.stdout, output.stdout);
}

/// A start renews this build's parser cache folder and removes the folders of other builds unused
/// for more than 30 days,
///  before any window:
///  here the display connection fails afterwards on
/// purpose,
///  and the cache is inspected.
#[test]
fn a_start_removes_cache_folders_of_other_builds_unused_for_30_days() {
    let fixture = tempfile::tempdir().expect("disposable CLI environment");
    let project = fixture.path().join("project");
    fs::create_dir(&project).expect("project fixture");
    let runtime = fixture.path().join("cache/monochromatic-ide/runtime");
    let now = SystemTime::now();
    for (key, days) in [("0000000000000031", 31), ("0000000000000001", 1)] {
        let folder = runtime.join(key);
        fs::create_dir_all(folder.join("grammars")).expect("seeded key folder");
        fs::write(folder.join("grammars/sql.so"), b"older build").expect("seeded library");
        fs::write(folder.join("last-used"), b"").expect("seeded marker");
        fs::File::options()
            .write(true)
            .open(folder.join("last-used"))
            .expect("marker")
            .set_modified(now - Duration::from_secs(days * 86_400))
            .expect("marker time");
    }
    let output = invoke(fixture.path(), &[project.as_os_str()]);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(&runtime).expect("runtime cache folder") {
        names.push(
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned(),
        );
    }
    names.sort();
    assert!(
        !runtime.join("0000000000000031").exists(),
        "the folder unused for 31 days remains: {names:?}\n{stderr}"
    );
    assert!(
        runtime.join("0000000000000001/grammars/sql.so").is_file(),
        "the folder used a day ago was removed: {names:?}\n{stderr}"
    );
    let mut current: Vec<&String> = Vec::new();
    for name in names.iter() {
        if name.as_str() != "0000000000000001" {
            current.push(name);
        }
    }
    assert_eq!(
        current.len(),
        1,
        "exactly this build's folder besides: {names:?}\n{stderr}"
    );
    let renewed = fs::symlink_metadata(runtime.join(current[0]).join("last-used"))
        .expect("this build's marker")
        .modified()
        .expect("marker time");
    assert!(
        renewed >= now - Duration::from_secs(60),
        "the start renewed this build's marker"
    );
}
