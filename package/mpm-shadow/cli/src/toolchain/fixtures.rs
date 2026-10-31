//! What:
//! A small plan and the lock it implies,
//! shaped like the repository's real ones (a subset covering the routes
//!           the repository uses),
//! for the unit tests of every toolchain module.
//! Why:
//! Tests need realistic input without reading the repository's real files or touching a real toolchain.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // fixtures.ts (test only)
//! export const SAMPLE_PLAN = `{ ... }`;
//! export function sampleLock(): Lock { ... }
//! ```

/// Imports what this module uses from `super::lock`.
use super::lock::{Lock, render_lock};
/// Imports what this module uses from `super::model`.
use super::model::{Artifact, LockedTool};
/// Imports what this module uses from `super::plan`.
use super::plan::{Plan, parse_plan};
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope, repo_scope};

/// The sample plan,
/// with comments and trailing commas the way people write it.
pub(crate) const SAMPLE_PLAN: &str = r#"{
  "schemaVersion": 1,
  "tools": {
    // The downloader the bin-sourced route needs first.
    "bin": {
      "request": "github:marcosnils/bin@0.29.4",
      "route": "archive",
      "bins": ["bin"],
    },
    "uv": {
      "request": "github:astral-sh/uv@latest",
      "route": "archive",
      "bins": ["uv", "uvx"],
    },
    "pnpm": {
      "request": "github:pnpm/pnpm@latest",
      "route": "bin-sourced",
      "bins": ["pnpm", "pn"],
      "options": { "binAliases": { "pn": "pnpm" } },
    },
    "node": {
      "request": "node@latest",
      "route": "archive",
      "bins": ["node", "npm", "npx"],
    },
    /* The rust toolchain: a channel, with components and targets. */
    "rust": {
      "request": "rust@nightly",
      "route": "rustup-toolchain",
      "manager": "rustup",
      "packageId": "nightly-2026-10-30",
      "bins": ["cargo", "rustc", "rustup"],
      "env": { "RUSTUP_TOOLCHAIN": "nightly-2026-10-30" },
      "options": {
        "rustComponents": ["clippy", "rust-src"],
        "rustTargets": ["aarch64-linux-android"],
      },
    },
    "cargo-binstall": {
      "request": "github:cargo-bins/cargo-binstall@latest",
      "route": "bin-sourced",
      "bins": ["cargo-binstall"],
    },
    "cargo:fd-find": {
      "request": "cargo:fd-find@latest",
      "route": "cargo-binstall",
      "manager": "cargo",
      "packageId": "fd-find",
      "bins": ["fd"],
    },
    "cargo:monochromatic-mpm-shadow": {
      "request": "cargo:monochromatic-mpm-shadow@latest",
      "route": "cargo-binstall",
      "manager": "cargo",
      "packageId": "monochromatic-mpm-shadow",
      "bins": ["mpm-shadow"],
      "os": ["linux"],
      "options": {
        "postinstall": {
          "when": "always",
          "link": { "sourceBin": "mpm-shadow", "linkName": "mpm", "linkDir": ".cache/mpm-shadow-bin" },
        },
      },
    },
    "npm:wrangler": {
      "request": "npm:wrangler@latest",
      "route": "mpm-pnpm",
      "manager": "pnpm",
      "packageId": "wrangler",
      "bins": ["wrangler"],
      "options": { "allowBuilds": ["esbuild"] },
    },
    "pipx:slopo": {
      "request": "pipx:slopo@latest",
      "route": "mpm-uvx",
      "manager": "uvx",
      "packageId": "slopo",
      "bins": ["slopo"],
    },
    "android-sdk": {
      "request": "android-sdk@latest",
      "route": "android-sdk",
      "bins": ["adb", "sdkmanager"],
      "env": { "ANDROID_HOME": "android-sdk/23.0" },
      "options": { "androidPackages": ["platforms/android-37.0", "platform-tools"] },
    },
  },
}
"#;

/// The 64-character hash the fixture lock gives every artifact row.
pub(crate) const SAMPLE_SHA256: &str = "439e9c438acdf6a6ffe47418da709153d66c4bed812e30a473faacb107dd2791";

/// The sample plan decoded.
pub(crate) fn sample_plan() -> Plan {
    return parse_plan(SAMPLE_PLAN, "sample plan").expect("the sample plan parses");
}

/// The version the fixture lock gives a tool.
pub(crate) fn sample_version(id: &str) -> String {
    return match id {
        "bin" => "0.29.4".to_string(),
        "rust" => "nightly-2026-10-30".to_string(),
        "android-sdk" => "23.0".to_string(),
        _ => "1.2.3".to_string(),
    };
}

/// The lock the sample plan implies:
/// the plan's declarations,
/// `sample_version` versions,
/// and one artifact row for the
/// archive and bin-sourced tools.
pub(crate) fn sample_lock() -> Lock {
    let plan: Plan = sample_plan();
    let mut tools: Vec<LockedTool> = Vec::new();
    for declaration in plan.tools {
        let needs_artifact: bool = matches!(
            declaration.route,
            super::model::Route::Archive | super::model::Route::BinSourced | super::model::Route::AndroidSdk
        );
        let mut artifacts: Vec<(String, Artifact)> = Vec::new();
        if needs_artifact {
            artifacts.push((
                "linux-x64".to_string(),
                Artifact {
                    url: format!("https://example.invalid/{}/download.tar.gz", declaration.id),
                    sha256: SAMPLE_SHA256.to_string(),
                },
            ));
        }
        let version: String = sample_version(&declaration.id);
        tools.push(LockedTool { declaration, version, artifacts });
    }
    return Lock { tools };
}

/// A disposable repository scope on disk:
/// the sample plan and the rendered sample lock written under a fresh
/// directory,
/// with a fake home inside it.
/// The returned guard removes everything when dropped.
pub(crate) fn sample_scope_on_disk() -> (tempfile::TempDir, Scope, EnvMap) {
    let directory = crate::test_scratch::scratch();
    let home = directory.path().join("home");
    let root = directory.path().join("work");
    std::fs::create_dir_all(&home).expect("make the home");
    std::fs::create_dir_all(&root).expect("make the worktree");
    let env = EnvMap::from_pairs(&[("HOME", home.to_str().expect("utf-8"))]);
    let scope = repo_scope(&root, &env).expect("a scope");
    std::fs::write(&scope.plan_file, SAMPLE_PLAN).expect("write the plan");
    std::fs::write(&scope.lock_file, render_lock(&sample_lock())).expect("write the lock");
    return (directory, scope, env);
}
