//! End-to-end test of the built `mpm-shadow` binary installing in a checkout that started as a copy of another one,
//! the way the git wrapper's `git worktree add` makes a new worktree:
//! a snapshot of the source checkout's ignored files,
//! `.cache` included,
//! then `mpm-shadow install --frozen-lockfile`.
//! The copy carries the source's record of postinstall links,
//! which names the source's links by absolute path.
//! An install in the copy must leave every link of the source in place.
//! The tree is a real (disposable) git repository with a second worktree;
//! the environment is cleared and the home is disposable,
//! so nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mpm_shadow::toolchain::lock::{parse_lock, render_lock};

/// The directory of the postinstall link in both checkouts,
/// relative to the checkout.
const LINK_DIRECTORY: &str = ".cache/monochromatic-git-policy-cli-bin";

/// The plan:
/// the git wrapper with the postinstall link the repository's plan gives it.
const PLAN: &str = r#"{
  "schemaVersion": 1,
  "tools": {
    "cargo:monochromatic-git-policy-cli": {
      "request": "cargo:monochromatic-git-policy-cli@latest",
      "route": "cargo-binstall",
      "packageId": "monochromatic-git-policy-cli",
      "bins": ["cli-git-native"],
      "os": ["linux"],
      "options": { "postinstall": { "when": "always", "link": { "sourceBin": "cli-git-native", "linkName": "git", "linkDir": ".cache/monochromatic-git-policy-cli-bin" } } },
    },
  },
}
"#;

/// A lock without a digest,
/// signed by the test with the library.
const UNSIGNED_LOCK: &str = r#"{
  "schemaVersion": 1,
  "generatedFrom": "package/file-enforcer/core/src/toolchain/plan.ts",
  "tools": {
    "cargo:monochromatic-git-policy-cli": {
      "request": "cargo:monochromatic-git-policy-cli@latest", "version": "0.4.5", "route": "cargo-binstall", "packageId": "monochromatic-git-policy-cli",
      "bins": ["cli-git-native"], "os": ["linux"], "env": {},
      "options": { "postinstall": { "when": "always", "link": { "sourceBin": "cli-git-native", "linkName": "git", "linkDir": ".cache/monochromatic-git-policy-cli-bin" } } },
      "artifacts": {}
    }
  }
}
"#;

/// A disposable home,
/// a git repository with a second worktree,
/// and a finished managed copy of the tool.
struct Fixture {
    /// The disposable directory everything sits in.
    directory: tempfile::TempDir,
}

/// The methods of `Fixture`.
impl Fixture {
    /// Builds the whole fixture.
    fn new() -> Fixture {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        let fixture = Fixture { directory };
        fs::create_dir_all(fixture.home()).expect("make the home");
        fs::create_dir_all(fixture.source()).expect("make the source checkout");
        fixture.git(&fixture.source(), &["init", "--quiet", "--initial-branch=main"]);
        fixture.git(&fixture.source(), &["worktree", "add", "--quiet", "-b", "two", &fixture.copy().to_string_lossy()]);
        let parsed = parse_lock(UNSIGNED_LOCK, "lock").expect("the fixture lock parses");
        for checkout in [fixture.source(), fixture.copy()] {
            fs::write(checkout.join("toolchain.plan.jsonc"), PLAN).expect("write the plan");
            fs::write(checkout.join("toolchain.lock.json"), render_lock(&parsed.lock)).expect("write the signed lock");
        }

        let managed_version = fixture.managed_root().join("0.4.5");
        let program = managed_version.join("bin").join("cli-git-native");
        fs::create_dir_all(program.parent().expect("a parent")).expect("make the directory");
        fs::write(&program, "#!/bin/sh\necho managed\n").expect("write the managed copy");
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
        fs::write(
            managed_version.join(".crates.toml"),
            "[v1]\n\"monochromatic-git-policy-cli 0.4.5 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"cli-git-native\"]\n",
        )
        .expect("write cargo's record for the managed copy");
        symlink("0.4.5", fixture.managed_root().join("current")).expect("point current at the managed copy");
        return fixture;
    }

    /// The disposable home.
    fn home(&self) -> PathBuf {
        return self.directory.path().join("home");
    }

    /// The checkout the copy is made from.
    fn source(&self) -> PathBuf {
        return self.directory.path().join("source");
    }

    /// The worktree that starts as a copy of the source.
    fn copy(&self) -> PathBuf {
        return self.directory.path().join("copy");
    }

    /// The managed crate directory (the version directories and `current`).
    fn managed_root(&self) -> PathBuf {
        return self.home().join(".local/share/monochromatic-dev/toolchain/cargo/monochromatic-git-policy-cli");
    }

    /// Runs the real `git` in a cleared environment.
    fn git(&self, directory: &Path, arguments: &[&str]) {
        let output = Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(arguments)
            .current_dir(directory)
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run git");
        assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    }

    /// Runs `mpm-shadow install --frozen-lockfile` in `checkout` with a cleared environment.
    fn install_in(&self, checkout: &Path) -> Output {
        return Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
            .args(["install", "--frozen-lockfile"])
            .current_dir(checkout)
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", "/usr/bin:/bin")
            // The fixture sits inside this repository's build directory: git must not find the repository above it.
            .env("GIT_CEILING_DIRECTORIES", self.directory.path())
            .output()
            .expect("run mpm-shadow");
    }

    /// Copies the source's `.cache` into the copy,
    /// symbolic links kept as links,
    /// as a snapshot of the source does.
    fn snapshot_cache_into_the_copy(&self) {
        let status = Command::new("cp")
            .arg("--archive")
            .arg(self.source().join(".cache"))
            .arg(self.copy())
            .status()
            .expect("start cp");
        assert!(status.success(), "cp copies the cache");
    }
}

/// Standard output and error as text.
fn text(output: &Output) -> String {
    return format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

#[test]
fn an_install_in_a_copy_of_a_checkout_leaves_that_checkouts_links_in_place() {
    let fixture = Fixture::new();
    let installed = fixture.install_in(&fixture.source());
    assert_eq!(installed.status.code(), Some(0), "{}", text(&installed));
    let source_link = fixture.source().join(LINK_DIRECTORY).join("git");
    let source_target = fs::read_link(&source_link).expect("the source checkout made its link");

    fixture.snapshot_cache_into_the_copy();
    let copied = fixture.install_in(&fixture.copy());
    assert_eq!(copied.status.code(), Some(0), "{}", text(&copied));

    assert_eq!(fs::read_link(&source_link).expect("the source checkout's link is still a link"), source_target, "{}", text(&copied));
    assert!(!text(&copied).contains("retired the postinstall link"), "{}", text(&copied));
    let copy_target = fs::read_link(fixture.copy().join(LINK_DIRECTORY).join("git")).expect("the copy made its own link");
    assert!(copy_target.starts_with(fixture.copy()), "the copy's link runs the copy's launcher: {}", copy_target.display());
}
