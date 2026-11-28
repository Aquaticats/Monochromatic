//! What:
//!  The built wrapper started the way a person's shell starts it.
//! Why:
//!  Every arm's subject is the command a person meets,
//!  so the words are the person's,
//!  the program is the one the driver built,
//!  and the environment names no session bus and no runtime directory of this
//!  account's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = arm.wrapperGit(['worktree', 'remove', shown(slot)]);
//! ```

use super::zfs_removal_force_guest_fixture::{
    Arm,
    WRAPPER,
};
use std::path::PathBuf;
use std::process::{
    Command,
    Output,
};

impl Arm {
    /// What:
    ///  Run one Git command through the built wrapper,
    ///  the way a person's shell starts it.
    /// Why:
    ///  The arm's subject is the command a person meets,
    ///  so the words are the person's and the wrapper is the program the driver
    ///  built.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// arm.wrapperGit(['worktree', 'remove', shown(slot)]);
    /// ```
    pub(super) fn wrapper_git(
        &self,
        arguments: &[&str],
    ) -> Output {
        let bin: PathBuf = self.root.join("bin");
        std::fs::create_dir_all(bin.as_path()).expect("the wrapper's bin folder");
        let home: PathBuf = self.root.join("home");
        std::fs::create_dir_all(home.as_path()).expect("the wrapper's home folder");
        let runtime: PathBuf = self.root.join("runtime");
        std::fs::create_dir_all(runtime.as_path()).expect("the wrapper's runtime folder");
        let link: PathBuf = bin.join("git");
        let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
        std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the wrapper's git link");
        return Command::new(link.as_path())
            .args(arguments)
            .current_dir(self.checkout.as_path())
            .env_clear()
            .env("PATH", format!("{}:{}", bin.display(), self.search_path()))
            .env("HOME", home.as_path())
            .env("XDG_RUNTIME_DIR", runtime.as_path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .output()
            .unwrap_or_else(|error| {
                panic!("the wrapper `git {arguments:?}` could not start: {error}")
            });
    }

    /// The wrapper's standard output and error,
    /// joined.
    pub(super) fn said(output: &Output) -> String {
        return format!(
            "{}{}",
            String::from_utf8_lossy(output.stdout.as_slice()),
            String::from_utf8_lossy(output.stderr.as_slice())
        );
    }
}
