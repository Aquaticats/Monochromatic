//! What:
//! Everything a test of a provider,
//! observer or resolver needs:
//! a disposable scope on disk,
//! the sample lock,
//! a fake runner and a fake network,
//! a Linux x64 machine to pretend to be,
//! and the stand-in program an install is given.
//! Why:
//! One constructor instead of the same dozen lines in every test file,
//! and the program is written through `fixture_programs`,
//! because a test that writes a program and starts it is refused with `ETXTBSY` when the test binary holds the
//! write open.
//! A caller imports from `fakes` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export class Rig {
//!   static new(): Rig;
//!   static throughSymlinkedHome(): [Rig, string];
//!   context(): Context;
//!   tool(id: string): LockedTool;
//!   installBinary(toolId: string, name: string): string;
//! }
//! export function writeExecutable(path: string): void;
//! ```

/// Imports the table-driven fake network this rig answers through.
use super::the_fake_network::FakeHttp;
/// Imports the recording fake runner this rig answers through.
use super::the_fake_runner::FakeRunner;
/// What:
/// Everything a test of a provider,
/// observer or resolver needs:
/// a disposable scope on disk,
/// the sample lock,
/// a
///           fake runner and a fake network,
/// and a Linux x64 host.
/// Why:
/// One constructor instead of the same twelve lines in every test file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Rig { scope; env; runner = new FakeRunner(); http = new FakeHttp(); lock = sampleLock(); host = linuxX64; }
/// ```
pub(crate) struct Rig {
    /// Keeps the disposable directory alive until the rig drops.
    pub(crate) _directory: tempfile::TempDir,
    /// The scope on disk.
    pub(crate) scope: super::super::scope::Scope,
    /// The fake environment (HOME inside the directory).
    pub(crate) env: super::super::scope::EnvMap,
    /// The recording fake runner.
    pub(crate) runner: FakeRunner,
    /// The table-driven fake network.
    pub(crate) http: FakeHttp,
    /// The locked tools the context sees.
    pub(crate) lock: super::super::lock::Lock,
    /// The pretended machine.
    pub(crate) host: super::super::context::Host,
}

/// The methods of `Rig`.
impl Rig {
    /// A rig with a fresh disposable scope,
    /// the sample lock and empty fakes.
    pub(crate) fn new() -> Rig {
        let (directory, scope, env) = super::super::fixtures::sample_scope_on_disk();
        return Rig {
            _directory: directory,
            scope,
            env,
            runner: FakeRunner::new(),
            http: FakeHttp::new(),
            lock: super::super::fixtures::sample_lock(),
            host: super::super::context::Host {
                platform: "linux-x64".to_string(),
                family: Some(super::super::model::OsFamily::Linux),
            },
        };
    }

    /// What:
    /// A rig whose `HOME` is spelled through a symbolic link,
    /// together with the home's canonical spelling.
    /// Why:
    /// A machine whose home is a symbolic link (Fedora Atomic's `/home` to `/var/home`) hands every program
    ///           a `HOME` that differs from the directory's physical path,
    /// which is the spelling that decides how pnpm writes its relative links.
    #[cfg(unix)]
    pub(crate) fn through_symlinked_home() -> (Rig, std::path::PathBuf) {
        let directory = crate::test_scratch::scratch();
        let physical = directory.path().join("physical");
        let physical_home = physical.join("home");
        let root = physical.join("work");
        std::fs::create_dir_all(&physical_home).expect("make the physical home");
        std::fs::create_dir_all(&root).expect("make the worktree");
        let link = directory.path().join("link");
        std::os::unix::fs::symlink(&physical, &link).expect("link the physical directory");
        let linked_home = link.join("home");
        let env = super::super::scope::EnvMap::from_pairs(&[(
            "HOME",
            linked_home.to_str().expect("utf-8"),
        )]);
        let scope = super::super::scope::repo_scope(&root, &env).expect("a scope");
        let canonical_home = std::fs::canonicalize(&physical_home).expect("the home exists");
        let rig = Rig {
            _directory: directory,
            scope,
            env,
            runner: FakeRunner::new(),
            http: FakeHttp::new(),
            lock: super::super::fixtures::sample_lock(),
            host: super::super::context::Host {
                platform: "linux-x64".to_string(),
                family: Some(super::super::model::OsFamily::Linux),
            },
        };
        return (rig, canonical_home);
    }

    /// A context borrowing this rig.
    pub(crate) fn context(&self) -> super::super::context::Context<'_> {
        return super::super::context::Context {
            scope: &self.scope,
            env: &self.env,
            runner: &self.runner,
            http: &self.http,
            host: &self.host,
            tools: &self.lock.tools,
        };
    }

    /// The locked tool with this id (cloned so the test can keep using the rig).
    pub(crate) fn tool(
        &self,
        id: &str,
    ) -> super::super::model::LockedTool {
        return self
            .lock
            .tool(id)
            .expect("the sample lock holds the tool")
            .clone();
    }

    /// Writes an executable stand-in at `<install dir of tool>/<name>`.
    pub(crate) fn install_binary(
        &self,
        tool_id: &str,
        name: &str,
    ) -> std::path::PathBuf {
        let tool = self.tool(tool_id);
        let path = super::super::layout::tool_install_dir(
            &self.scope.toolchain_root,
            tool_id,
            &tool.version,
        )
        .expect("a plain directory name")
        .join(name);
        write_executable(&path);
        return path;
    }
}

/// Writes a small executable file,
/// creating its directory.
/// The write belongs to a child process:
/// `fixture_programs` says why.
pub(crate) fn write_executable(path: &std::path::Path) {
    super::super::fixture_programs::write_program(path, b"#!/bin/sh\n");
}
