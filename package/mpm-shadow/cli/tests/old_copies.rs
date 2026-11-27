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
//! This file holds the disposable tree every test runs against;
//! each test is a file of its own beside it:
//! every link moving before the copy goes (`every_link_moves_and_the_old_copy_goes`),
//! the two `--lockfile-only` modes touching nothing (`lockfile_only_touches_no_link`),
//! and a tree git cannot list keeping the copy (`a_tree_git_cannot_list_keeps_the_copy`).
//! Why:
//! One fixture means the tests cannot disagree about what the binary sees.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // old_copies.test.ts
//! import { Fixture, makeLink, text, writeScript } from './old_copies/fixture.ts';
//! await import('./old_copies/every_link_moves_and_the_old_copy_goes.ts');
//! // ...one import per test...
//! ```

use std::fs;
use std::os::unix::fs::{
    PermissionsExt,
    symlink,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Output,
};

use mpm_shadow::toolchain::lock::{
    parse_lock,
    render_lock,
};

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
        let directory =
            tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        let fixture = Fixture {
            directory,
        };
        fs::create_dir_all(fixture.home()).expect("make the home");
        fs::create_dir_all(fixture.work()).expect("make the worktree");
        fixture.git(
            &fixture.work(),
            &["init", "--quiet", "--initial-branch=main"],
        );
        fixture.git(
            &fixture.work(),
            &[
                "worktree",
                "add",
                "--quiet",
                "-b",
                "two",
                &fixture.work_two().to_string_lossy(),
            ],
        );
        fs::write(fixture.work().join("toolchain.plan.jsonc"), PLAN).expect("write the plan");
        let parsed = parse_lock(UNSIGNED_LOCK, "lock").expect("the fixture lock parses");
        fs::write(
            fixture.work().join("toolchain.lock.json"),
            render_lock(&parsed.lock),
        )
        .expect("write the signed lock");

        write_script(&fixture.old_copy(), "echo old");
        let managed_version = fixture.managed_root().join("0.4.5");
        write_script(
            &managed_version.join("bin").join("cli-git-native"),
            "echo managed",
        );
        fs::write(
            managed_version.join(".crates.toml"),
            "[v1]\n\"monochromatic-git-policy-cli 0.4.5 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"cli-git-native\"]\n",
        )
        .expect("write cargo's record for the managed copy");
        symlink("0.4.5", fixture.managed_root().join("current"))
            .expect("point current at the managed copy");
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
        return self
            .home()
            .join(".cargo")
            .join("bin")
            .join("cli-git-native");
    }

    /// The managed crate directory (the version directories and `current`).
    fn managed_root(&self) -> PathBuf {
        return self
            .home()
            .join(".local/share/monochromatic-dev/toolchain/cargo/monochromatic-git-policy-cli");
    }

    /// Runs the real `git` in a cleared environment.
    fn git(
        &self,
        directory: &Path,
        arguments: &[&str],
    ) {
        let output = Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(arguments)
            .current_dir(directory)
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Runs the built binary in the first checkout with a cleared environment.
    fn run(
        &self,
        arguments: &[&str],
    ) -> Output {
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
fn write_script(
    path: &Path,
    body: &str,
) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("make the directory");
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("write the script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// Points `link` at `target`,
/// making the link's directory.
fn make_link(
    link: &Path,
    target: &Path,
) {
    fs::create_dir_all(link.parent().expect("a parent")).expect("make the directory");
    symlink(target, link).expect("make the link");
}

/// Standard output as text.
fn text(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).to_string();
}

/// The run that moves every link before the old copy goes.
#[path = "old_copies/every_link_moves_and_the_old_copy_goes.rs"]
mod every_link_moves_and_the_old_copy_goes;

/// The runs that show the two `--lockfile-only` modes touch no link and remove no copy.
#[path = "old_copies/lockfile_only_touches_no_link.rs"]
mod lockfile_only_touches_no_link;

/// The run that shows a tree git cannot list keeps the old copy and says why.
#[path = "old_copies/a_tree_git_cannot_list_keeps_the_copy.rs"]
mod a_tree_git_cannot_list_keeps_the_copy;
