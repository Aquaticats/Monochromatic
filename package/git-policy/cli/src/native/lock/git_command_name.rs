//! What:
//!  Whether a process's command name is Git's,
//!  or one of its helpers'.
//! Why:
//!  Two judgements ask the same question of `/proc/<pid>/comm`:
//!  whether the process a lock's PID file names could be the program that wrote it,
//!  of a foreign `index.lock` and of a reference lock.
//!  Git writes locks from binaries called `git` and from helpers such as `git-remote-https`,
//!  so the first characters are what is compared,
//!  and one spelling of the prefix keeps the two answers the same.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const namesGitCommand = (name: string): boolean => name.startsWith('git');
//! ```

/// The prefix of Git's command name and of its helpers (`git-remote-https`),
/// as the kernel reports it in `/proc/<pid>/comm`.
const GIT_COMMAND_PREFIX: &str = "git";

/// Whether a command name starts with Git's,
/// which is what the kernel reports of a Git binary and of its helpers.
pub fn names_git_command(name: &str) -> bool {
    return name.starts_with(GIT_COMMAND_PREFIX);
}

/// Controls on the command-name test stay out of the release executable.
#[cfg(test)]
#[path = "git_command_name_tests.rs"]
mod tests;
