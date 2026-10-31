//! What:
//! The bundle every provider,
//! observer and resolver receives:
//! the scope,
//! the environment,
//! the program runner,
//!           the HTTP client,
//! the host description and the locked tools.
//! Why:
//! One value carries every seam,
//! so a test builds a context from fakes and a disposable scope and no
//!           function reaches for the real process,
//! network or home directory on its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // context.ts
//! export type Context = { scope: Scope; env: EnvMap; runner: Runner; http: HttpGet; host: Host; tools: LockedTool[] };
//! ```

/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `HttpGet` from `super::http`.
use super::http::HttpGet;
/// Imports the tool lookup.
use super::util::find_tool;
/// Imports what this module uses from `super::layout`.
use super::layout::{current_family, current_platform_id, find_tool_bin};
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, OsFamily};
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest, Runner, toolchain_path_env};
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope};

/// What:
/// The machine the commands run on,
/// as the lock names it:
/// the platform id (`linux-x64`) and the OS family.
/// Why:
/// Injected so tests exercise `linux-arm64-musl` or `macos-arm64` on any host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Host = { platform: string; family?: OsFamily };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Host {
    /// The key of the host's artifact row in the lock.
    pub platform: String,
    /// The OS family,
    /// or `None` for a family the plan cannot name.
    pub family: Option<OsFamily>,
}

/// The methods of `Host`.
impl Host {
    /// The running machine.
    pub fn current() -> Host {
        return Host { platform: current_platform_id(), family: current_family() };
    }
}

/// Everything a provider,
/// observer or resolver needs;
/// borrowed,
/// never owned,
/// so building one is free.
#[derive(Clone, Copy)]
pub struct Context<'a> {
    /// The scope (paths).
    pub scope: &'a Scope,
    /// The environment variables.
    pub env: &'a EnvMap,
    /// Starts programs.
    pub runner: &'a dyn Runner,
    /// Reaches the network.
    pub http: &'a dyn HttpGet,
    /// The machine.
    pub host: &'a Host,
    /// Every locked tool of the scope (providers look up prerequisites such as `uv` and `pnpm` here).
    pub tools: &'a [LockedTool],
}

/// The methods of `Context<'_>`.
impl Context<'_> {
    /// The locked tool with this id,
    /// or an error naming the id.
    pub fn tool(&self, id: &str) -> Result<&LockedTool, ToolchainError> {
        return match find_tool(self.tools, id) {
            Some(found) => Ok(found),
            None => Err(ToolchainError::failure(format!("the toolchain plan has no tool {id:?}"))),
        };
    }

    /// The executable `name` of the tool `tool_id`,
    /// searched where that tool's route installs it,
    /// or `None`.
    pub fn find_bin_of(&self, tool_id: &str, name: &str, extra_dirs: &[PathBuf]) -> Option<PathBuf> {
        let tool: &LockedTool = self.tool(tool_id).ok()?;
        return find_tool_bin(tool, name, self.scope, self.env, extra_dirs);
    }

    /// The environment installs run with (see `toolchain_path_env`).
    pub fn path_env(&self) -> BTreeMap<String, String> {
        return toolchain_path_env(self.scope, self.env);
    }

    /// A request for `command` run with the toolchain environment.
    pub fn request(&self, command: &str, args: &[&str]) -> RunRequest {
        return RunRequest::new(command, args).with_env(self.path_env());
    }

    /// Runs a request.
    pub fn run(&self, request: &RunRequest) -> RunOutcome {
        return self.runner.run(request);
    }

    /// What:
    /// The GitHub token for API calls and `bin` downloads:
    /// `GITHUB_TOKEN`,
    /// else the output of
    ///           `gh auth token`,
    /// else `None`.
    /// Why:
    /// Unauthenticated GitHub API calls run out of quota in the middle of a sync or an update.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// resolveGithubToken(): Promise<string | undefined>
    /// ```
    pub fn github_token(&self) -> Option<String> {
        if let Some(token) = self.env.get("GITHUB_TOKEN") {
            return Some(token.to_string());
        }
        let outcome: RunOutcome = self.runner.run(&RunRequest::new("gh", &["auth", "token"]).with_env(self.env.all().clone()));
        let token: String = outcome.stdout.trim().to_string();
        if outcome.succeeded() && !token.is_empty() {
            return Some(token);
        }
        return None;
    }
}
