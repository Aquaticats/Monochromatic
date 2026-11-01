//! What:
//!  Controls of the IDE and Android preparations.
//! Why:
//!  Both change how the package compiles,
//! so the artifact requirement,
//! the links,
//! the parsing of the NDK variables
//!  and the failure of the NDK query each need a case.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('parses exports', () => expect(parseExports('export A="1"')).toEqual([['A', '1']]));
//! ```

use super::{android_setup, ide_setup, ndk_home_of, parse_exports};
use crate::{closure_stub_support::scenario, test_fs::Fixture};
use std::path::{Path, PathBuf};

/// Write the artifacts that the IDE's runtime and notices tasks leave under `target`.
fn prepare_artifacts(package: &Path) {
    let runtime: PathBuf = package.join("target").join("debug").join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime directory");
    std::fs::write(runtime.join("manifest.json"), "{}").expect("manifest");
    std::fs::write(package.join("target").join("crate-licenses.json"), "[]").expect("licenses");
}

/// Export lines are read;
/// anything else is ignored.
#[test]
fn parses_only_well_formed_export_lines() {
    let text: &str = "export A=\"1\"\n\
        export B_2=\"two words\"\n\
        export C=unquoted\n\
        export D=\"unterminated\n\
        export bad-name=\"x\"\n\
        export =\"empty name\"\n\
        # To import with bash/zsh/etc:\n\
        #     source <(cargo ndk-env)\n\
        \n\
        export E=\"\"\n\
        A=\"not exported\"\n";
    assert_eq!(
        parse_exports(text),
        vec![
            (String::from("A"), String::from("1")),
            (String::from("B_2"), String::from("two words")),
            (String::from("E"), String::new()),
        ]
    );
}

/// A value with shell metacharacters stays data.
#[test]
fn a_value_with_shell_syntax_is_kept_as_text() {
    assert_eq!(
        parse_exports("export X=\"$(touch /tmp/x); `id`\"\n"),
        vec![(String::from("X"), String::from("$(touch /tmp/x); `id`"))]
    );
}

/// Without the artifacts the IDE is skipped with a reason that names the tasks and the files.
#[test]
fn the_ide_without_artifacts_is_skipped() {
    let fixture: Fixture = Fixture::new();
    let reason: String = ide_setup(&fixture.path).expect_err("artifacts are missing");
    assert!(reason.contains("runtime and notices"), "{reason}");
    assert!(
        reason.contains("manifest.json") && reason.contains("crate-licenses.json"),
        "{reason}"
    );
    prepare_artifacts(&fixture.path);
    std::fs::remove_file(fixture.path.join("target").join("crate-licenses.json"))
        .expect("remove licenses");
    assert!(ide_setup(&fixture.path).is_err());
}

/// With the artifacts,
/// the build directory gets both links and is selected;
/// a second call replaces the links.
#[cfg(unix)]
#[test]
fn the_ide_with_artifacts_links_them_into_the_build_directory() {
    let fixture: Fixture = Fixture::new();
    let package: PathBuf = fixture.path.join("ide package");
    prepare_artifacts(&package);
    for _ in 0..2 {
        let variables = ide_setup(&package).expect("prepared");
        let build: PathBuf = package.join("target").join("dylint");
        assert_eq!(
            variables,
            vec![(
                String::from("CARGO_BUILD_BUILD_DIR"),
                build.to_string_lossy().into_owned()
            )]
        );
        let runtime_link: PathBuf = build.join("debug").join("build").join("runtime");
        assert!(
            runtime_link.join("manifest.json").is_file(),
            "the runtime link resolves"
        );
        assert!(
            build.join("debug").join("crate-licenses.json").is_file(),
            "the licenses link resolves"
        );
        assert!(
            runtime_link
                .symlink_metadata()
                .expect("link")
                .file_type()
                .is_symlink()
        );
    }
}

/// A real file where a link belongs is replaced by the link.
#[cfg(unix)]
#[test]
fn a_stale_file_in_place_of_a_link_is_replaced() {
    let fixture: Fixture = Fixture::new();
    prepare_artifacts(&fixture.path);
    let place: PathBuf = fixture.path.join("target").join("dylint").join("debug");
    std::fs::create_dir_all(&place).expect("build directory");
    std::fs::write(place.join("crate-licenses.json"), "stale").expect("stale file");
    ide_setup(&fixture.path).expect("prepared");
    assert!(
        place
            .join("crate-licenses.json")
            .symlink_metadata()
            .expect("link")
            .file_type()
            .is_symlink()
    );
}

/// The NDK variables and the target argument come from the `cargo ndk-env` answer,
///  and the NDK home is derived from its sysroot unless the machine owner exported one.
#[test]
fn android_setup_takes_the_variables_and_adds_the_target() {
    let stub = scenario(&[(
        "ndk-env.stdout",
        "export CC_aarch64_linux_android=\"/ndk dir/clang\"\nexport CARGO_NDK_SYSROOT_PATH=\"/sdk dir/ndk/29.0/toolchains/llvm/prebuilt/linux-x86_64/sysroot\"\n",
    )]);
    let setup = android_setup(&stub.cargo).expect("prepared");
    assert_eq!(
        setup.environment[0],
        (
            String::from("CC_aarch64_linux_android"),
            String::from("/ndk dir/clang")
        )
    );
    let mut derived: Option<(String, String)> = None;
    for pair in &setup.environment {
        if pair.0 == "ANDROID_NDK_HOME" {
            derived = Some(pair.clone());
        }
    }
    if std::env::var_os("ANDROID_NDK_HOME").is_none() {
        assert_eq!(
            derived,
            Some((
                String::from("ANDROID_NDK_HOME"),
                String::from("/sdk dir/ndk/29.0")
            ))
        );
    } else {
        assert_eq!(
            derived, None,
            "an exported ANDROID_NDK_HOME is kept, not replaced"
        );
    }
    assert_eq!(
        setup.arguments,
        vec![
            String::from("--target"),
            String::from("aarch64-linux-android")
        ]
    );
    assert_eq!(
        stub.calls(),
        vec![String::from("ndk-env|ndk-env --target arm64-v8a|")]
    );
}

/// A failing query,
/// an empty answer and a missing program each skip with a reason.
#[test]
fn android_setup_failures_are_skips() {
    let failing = scenario(&[("ndk-env.status", "1"), ("ndk-env.stderr", "no NDK here")]);
    let reason: String = android_setup(&failing.cargo).expect_err("failed query");
    assert!(reason.contains("no NDK here"), "{reason}");
    let empty = scenario(&[]);
    assert!(
        android_setup(&empty.cargo)
            .expect_err("empty answer")
            .contains("printed no variable")
    );
    let missing = android_setup(Path::new("/nonexistent/monochromatic-lint/cargo"))
        .expect_err("cargo missing");
    assert!(missing.contains("install cargo-ndk"), "{missing}");
}

/// The NDK directory is the part of the sysroot path before `/toolchains/`.
#[test]
fn the_ndk_home_is_the_part_before_the_toolchains_directory() {
    assert_eq!(
        ndk_home_of("/sdk dir/ndk/29.0/toolchains/llvm/prebuilt/linux-x86_64/sysroot"),
        Some(String::from("/sdk dir/ndk/29.0"))
    );
    assert_eq!(ndk_home_of("/sdk/ndk/29.0/sysroot"), None);
    assert_eq!(ndk_home_of("/toolchains/llvm"), None);
    assert_eq!(ndk_home_of(""), None);
}
