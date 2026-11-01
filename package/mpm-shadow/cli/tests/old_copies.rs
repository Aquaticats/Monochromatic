//! End-to-end tests of the built `mpm-shadow` binary retiring an old copy of this repository's own tool:
//! the git wrapper in the shared cargo bin directory,
//! the links to it in `~/.local/bin` and in a second checkout,
//! and a foreign link and a regular file that must stay.
//! The tree is a real (disposable) git repository with a second worktree,
//! so the binary asks the real `git` for the checkouts;
//! the environment is cleared and the home is disposable,
//! so nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.
//! No cargo is on `PATH`,
//! and the fixture has no cargo record for the copy,
//! so the removal takes the direct path (the `cargo uninstall` path is covered by the unit tests and by a run of
//! the binary against a disposable cargo home).

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mpm_shadow::toolchain::lock::{parse_lock, render_lock};

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
      "options": { "postinstall": { "when": "always", "link": { "sourceBin": "cli-git-native", "linkName": "git", "linkDir": ".cache/cli-git-bin" } } },
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
      "options": { "postinstall": { "when": "always", "link": { "sourceBin": "cli-git-native", "linkName": "git", "linkDir": ".cache/cli-git-bin" } } },
      "artifacts": {}
    }
  }
}
"#;

/// A disposable home,
/// a git repository with a second worktree,
/// and the machine the owner has:
/// an old copy,
/// a finished managed copy,
/// and links to the old copy.
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
        fs::create_dir_all(fixture.work()).expect("make the worktree");
        fixture.git(&fixture.work(), &["init", "--quiet", "--initial-branch=main"]);
        fixture.git(&fixture.work(), &["worktree", "add", "--quiet", "-b", "two", &fixture.work_two().to_string_lossy()]);
        fs::write(fixture.work().join("toolchain.plan.jsonc"), PLAN).expect("write the plan");
        let parsed = parse_lock(UNSIGNED_LOCK, "lock").expect("the fixture lock parses");
        fs::write(fixture.work().join("toolchain.lock.json"), render_lock(&parsed.lock)).expect("write the signed lock");

        write_script(&fixture.old_copy(), "echo old");
        let managed_version = fixture.managed_root().join("0.4.5");
        write_script(&managed_version.join("bin").join("cli-git-native"), "echo managed");
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

    /// The first checkout.
    fn work(&self) -> PathBuf {
        return self.directory.path().join("work");
    }

    /// The second worktree of the repository.
    fn work_two(&self) -> PathBuf {
        return self.directory.path().join("work-two");
    }

    /// The old copy in the shared cargo bin directory.
    fn old_copy(&self) -> PathBuf {
        return self.home().join(".cargo").join("bin").join("cli-git-native");
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

    /// Runs the built binary in the first checkout with a cleared environment.
    fn run(&self, arguments: &[&str]) -> Output {
        return Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
            .args(arguments)
            .current_dir(self.work())
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", "/usr/bin:/bin")
            // The fixture sits inside this repository's build directory: git must not find the repository above it.
            .env("GIT_CEILING_DIRECTORIES", self.directory.path())
            .output()
            .expect("run mpm-shadow");
    }
}

/// Writes an executable shell script,
/// making its directory.
fn write_script(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("make the directory");
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("write the script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// Points `link` at `target`,
/// making the link's directory.
fn make_link(link: &Path, target: &Path) {
    fs::create_dir_all(link.parent().expect("a parent")).expect("make the directory");
    symlink(target, link).expect("make the link");
}

/// Standard output as text.
fn text(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).to_string();
}

#[test]
fn install_moves_every_link_to_the_managed_copy_then_removes_the_old_one_and_a_second_run_does_nothing() {
    let fixture = Fixture::new();
    let local_git = fixture.home().join(".local/bin/git");
    let second_checkout_git = fixture.work_two().join(".cache/cli-git-bin/git");
    let foreign_target = fixture.home().join(".local/share/other/tool");
    write_script(&foreign_target, "echo foreign");
    let foreign_link = fixture.home().join(".local/bin/foreign");
    write_script(&fixture.home().join(".local/bin/not-a-link"), "echo regular");
    make_link(&local_git, &fixture.old_copy());
    make_link(&second_checkout_git, &fixture.old_copy());
    make_link(&foreign_link, &foreign_target);

    let listed = fixture.run(&["list"]);
    assert_eq!(listed.status.code(), Some(0), "an old copy is not drift: {}", text(&listed.stdout));
    assert!(text(&listed.stdout).contains("would be removed by `install` after it did: re-point 2 link(s)"), "{}", text(&listed.stdout));
    assert!(fixture.old_copy().exists(), "list changes nothing");

    let installed = fixture.run(&["install", "--frozen-lockfile"]);
    assert_eq!(installed.status.code(), Some(0), "{}{}", text(&installed.stdout), text(&installed.stderr));
    let managed = fixture.managed_root().join("current/bin/cli-git-native");
    for link in [&local_git, &second_checkout_git] {
        assert_eq!(fs::read_link(link).expect("still a link"), managed, "{}", link.display());
    }
    assert!(fs::symlink_metadata(fixture.old_copy()).is_err(), "the old copy is gone: {}", text(&installed.stdout));
    assert_eq!(fs::read_link(&foreign_link).expect("a link"), foreign_target, "a foreign link is untouched");
    assert!(fixture.home().join(".local/bin/not-a-link").is_file(), "a regular file is untouched");
    let output = text(&installed.stdout);
    assert_eq!(output.matches("toolchain install: re-pointed ").count(), 2, "{output}");
    assert!(output.contains(&format!("toolchain install: removed {}", fixture.old_copy().display())), "{output}");
    let through_the_link = Command::new(&local_git).env_clear().output().expect("run git through the re-pointed link");
    assert_eq!(text(&through_the_link.stdout).trim(), "managed", "git now runs the managed copy");

    let again = fixture.run(&["install", "--frozen-lockfile"]);
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    assert!(!text(&again.stdout).contains("old copy") && !text(&again.stdout).contains("re-pointed"), "{}", text(&again.stdout));
    assert!(!text(&fixture.run(&["list"]).stdout).contains("old copy"));
}

#[test]
fn install_lockfile_only_changes_no_link_and_removes_no_copy() {
    let fixture = Fixture::new();
    let local_git = fixture.home().join(".local/bin/git");
    make_link(&local_git, &fixture.old_copy());
    let locked = fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]);
    assert_eq!(locked.status.code(), Some(0), "{}", text(&locked.stderr));
    let offline = fixture.run(&["install", "--lockfile-only", "--offline"]);
    assert_eq!(offline.status.code(), Some(0), "{}", text(&offline.stderr));
    assert!(fixture.old_copy().exists());
    assert_eq!(fs::read_link(&local_git).expect("a link"), fixture.old_copy());
}

#[test]
fn a_tree_git_cannot_list_keeps_the_old_copy_and_says_why() {
    let fixture = Fixture::new();
    let local_git = fixture.home().join(".local/bin/git");
    make_link(&local_git, &fixture.old_copy());
    fs::remove_dir_all(fixture.work().join(".git")).expect("make the tree not a repository");
    let installed = fixture.run(&["install", "--frozen-lockfile"]);
    assert_eq!(installed.status.code(), Some(0), "{}{}", text(&installed.stdout), text(&installed.stderr));
    assert!(fixture.old_copy().exists(), "a checkout's link could be hiding where git would not tell");
    assert!(text(&installed.stdout).contains("is kept"), "{}", text(&installed.stdout));
    assert_eq!(fs::read_link(&local_git).expect("a link"), fixture.managed_root().join("current/bin/cli-git-native"), "the link that was seen moved");
}
