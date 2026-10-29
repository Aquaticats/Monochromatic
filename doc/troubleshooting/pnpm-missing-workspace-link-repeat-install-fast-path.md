# pnpm 12.16.1: a plain `pnpm install` prints `Already up to date` and leaves a missing declared workspace link missing

When the manifests and lockfile still match the digests in `node_modules/.pnpm-workspace-state-v1.json`,
pnpm's repeat-install fast path returns before any linking.
Its only link check treats a link that exists but points nowhere as stale,
and treats an absent link as healthy,
so a declared workspace dependency whose link was never created (or was deleted) is not repaired by `pnpm install`.
`pnpm install --force` repairs it.

A sibling investigation of the same family,
where a repair install did create a missing link while still printing `Already up to date`,
is [pnpm-stale-node-modules-detection.md](pnpm-stale-node-modules-detection.md) (pnpm 12.5.1).

## Symptom

In this repository on 2026-10-29,
`package/file-enforcer/core/package.json` declared `@monochromatic-dev/config-typescript` as a `workspace:*` dev dependency,
the lockfile recorded it for that importer,
and the package existed at `package/config/typescript`,
but `package/file-enforcer/core/node_modules/@monochromatic-dev/` had no `config-typescript` entry
(the neighbouring `config-rolldown` link was there).
Two sessions expected `pnpm install` to create it.
`mise run //:prepare:pnpm:install` (which runs `pnpm install`) exited 0 in 9.7 s with:

```text
Scope: all 224 workspace projects
Lockfile passes supply-chain policies (verified 18m ago)
Lockfile is up to date, resolution step is skipped
Already up to date
Done in 9.7s using pnpm v12.16.1
```

and the link was still missing afterwards.
Nothing failed:
`tsconfig.json` extends `@monochromatic-dev/config-typescript/dom`,
and Node resolution walked up to the root `node_modules/@monochromatic-dev/config-typescript`,
a valid symlink to `package/config/typescript`.

## Root cause

The fast path is `check_optimistic_repeat_install` in `pnpm/crates/package-manager/src/optimistic_repeat_install.rs`.
Its module comment (lines 4-9) says what it compares:

```rust
//! The install logs "Already up to date" when nothing has changed,
//! before any of the install setup runs.
//! The check compares the content
//! digests [`validation`] recorded in
//! `<workspace_root>/node_modules/.pnpm-workspace-state-v1.json` with the
//! manifests and the lockfile,
//! without parsing the lockfile or touching the
//! verifier cache or resolver state while every project is still proven
//! installed.
```

After the settings,
catalog and project-structure comparisons,
`freshness.rs:257-265` runs the two filesystem checks that "prove installed":

```rust
    if !project_structure_matches(state, project_manifests.as_slice()) {
        return Err(Stale::ProjectStructure);
    }
    if let Some(id) = first_held_project_missing_modules_dir(check, state, scope) {
        return Err(Stale::MissingModulesDir(id));
    }
    if policy.check_dangling_direct_links && direct_dependency_link_dangling(check) {
        return Err(Stale::DanglingDirectLink);
    }
    Ok(())
```

The first check asks whether a project's modules directory exists at all.
The second,
`modules_dirs.rs:164-207`,
asks about links,
and its documentation excludes the missing case on purpose:

```rust
/// Whether a direct dependency's entry in its project's modules directory is
/// a link whose target no longer exists.
/// Nothing the fast path records moves
/// when a link is broken or retargeted outside pnpm,
/// and the full install
/// relinks it.
/// A missing entry is not a broken link:
/// skipped optional and
/// excluded dependencies have none,
/// and a healthy entry costs one `stat`.
```

```rust
fn is_dangling_link(path: &Path) -> bool {
    fs::metadata(path).is_err() && fs::symlink_metadata(path).is_ok()
}
```

`is_dangling_link` is true only when the path exists as a symlink (`symlink_metadata` is `Ok`) whose target does not (`metadata` is `Err`).
An absent entry makes both calls fail,
so it is not dangling and the check passes.
When no digest,
setting or project structure changed,
`Decision::UpToDate` follows and the pipeline,
including linking,
never runs.

So the behavior is deliberate for optional and excluded dependencies,
and the gap is that a declared,
non-optional workspace dependency has no skip reason yet gets the same pass.

How the state file ended up recording digests of the manifest that already declared `config-typescript`,
without a link,
in this repository is not established here.
The sequence the earlier doc documents (`pnpm install --lockfile-only` refreshing digests without touching `node_modules`) is one way,
but this repository's history of that directory was not reconstructed.

Source under citation:
`pnpm/pnpm` shallow clone of the default branch at commit `f7d7e6173ad68ee44918173bdded12602966cb80` (2026-10-29).
The binary under test is the installed `pnpm` 12.16.1;
the clone has no tags,
so these line numbers were not checked against the 12.16.1 release itself.

## Verification

Version under test:
`pnpm` 12.16.1 (`pnpm --version`).

The harness is a throwaway workspace under `${HOME}/temp/agent/`,
with its own `HOME`,
no registry access (`--offline`),
and no repository state of this project:

```bash
root="$(mktemp --directory "${HOME}/temp/agent/pnpm-harness-XXXXXXXX")"
cd "${root}"
printf 'packages:\n  - packages/*\n' > pnpm-workspace.yaml
printf '{"name":"root","private":true,"version":"1.0.0"}' > package.json
mkdir --parents packages/a packages/b
printf '{"name":"b","version":"1.0.0"}' > packages/b/package.json
printf '{"name":"a","version":"1.0.0"}' > packages/a/package.json
pnpm install --offline
# Declare the workspace dependency and install: the link appears.
printf '{"name":"a","version":"1.0.0","devDependencies":{"b":"workspace:*"}}' > packages/a/package.json
pnpm install --offline
# Delete the link, then repeat the install.
rm packages/a/node_modules/b
pnpm install --offline
pnpm install --offline --force
```

Output recorded on 2026-10-29 (each step's link check is whether `packages/a/node_modules/b` exists):

```text
pnpm version: 12.16.1
install 1 exit 0 (a has no dependencies yet)
install 2 (declares b) exit 0 link a/node_modules/b exists: true
after removing the link, exists: false
install 3 exit 0 output: Scope: all 3 workspace projects | Already up to date | Done in 7ms using pnpm v12.16.1 link exists: false
install --force exit 0 link exists: true
```

### Patterns that work

- The first install after a manifest change (install 2):
  the content digests differ from the state file,
  the fast path declines,
  and the link is created.
- `pnpm install --force`:
  the link is back.

### Patterns that fail

- A repeat install with unchanged manifests and a deleted or never-created declared link:
  `Already up to date`,
  exit 0,
  link still absent.

### Not tested

- Whether a narrower trigger than `--force` repairs it (for example touching the manifest so its digest changes);
  only `--force` was run.
- The same sequence with the isolated linker settings of this repository's own `pnpm-workspace.yaml`;
  the harness uses defaults.

## Verified workarounds

`pnpm install --force`,
as in the harness,
restores the link.

Tradeoffs:

- It re-links the whole workspace,
  which for this repository means 224 projects in a `node_modules` that other agent sessions use at the same time,
  so it was not run here.
- This repository's task runner has no task for it;
  `mise run //:prepare:pnpm:install` runs plain `pnpm install`,
  and raw `pnpm` is not allowed for agents.
- Creating the one symlink by hand fixes the symptom but is outside the package manager and leaves the state file unchanged.

For this repository's case no repair was applied:
the link is not needed,
because Node resolution reaches the root link.

## What does not work

- Re-running `mise run //:prepare:pnpm:install`:
  it goes through the same fast path,
  with the same output as above.
- Assuming a repair install always relinks,
  as the earlier doc's 12.5.1 sequence did:
  that sequence had changed manifests since the last validation,
  so the fast path declined;
  with unchanged manifests it does not.

## Upstream filing decision

Walk of the six constraints,
with the default policy of not filing.

1. Is it really upstream's fault?
   Partly.
   The behavior follows a documented choice (`modules_dirs.rs:167-168`:
   a missing entry is not a broken link because skipped optional and excluded dependencies have none).
   The gap is that a declared non-optional workspace dependency is treated the same,
   so the message `Already up to date` is wrong for that case.
   This is a design trade-off with a narrower correct behavior,
   not a plain bug.
2. Can upstream fix it?
   Yes in principle:
   the check could ask for a link only for dependencies that are neither optional nor skipped.
   No architectural blocker was found.
3. Are they supporting this use case?
   Workspace links between projects are a core feature,
   and `repeat_install.rs` and `relocated_install.rs` under `pnpm/crates/cli/tests/suite/` test repeat-install behavior,
   so a link-repair expectation is plausible but was not confirmed in those tests.
4. Would the repository welcome our contribution?
   `CONTRIBUTING.md` (section "AI-assisted contributions",
   lines 325-345) welcomes agent-assisted contributions with a disclosure footer such as `Written by an agent (Claude Code, claude-opus-4-7).`,
   and warns that unvetted agent output,
   duplicate PRs and PRs that do not compile may be closed.
   No ban found.
5. Will they likely fix it?
   No signal either way.
   `gh search issues --repo pnpm/pnpm` for `missing workspace link Already up to date` and for `repeat install missing symlink node_modules not relinked` each returned no issues (checked 2026-10-29).
6. Have we prototyped a minimal fix compatible with their architecture?
   Not yet.
   The prototype is a change to `direct_dependency_link_dangling` (or a sibling check next to it) plus a test in `pnpm/crates/cli/tests/suite/repeat_install.rs`,
   built and run in a secret-free container;
   it was not attempted in this session because building the Rust workspace is heavy and outside the investigation that produced this doc.

Decision:
do not file as-is.
The audit is incomplete,
not closed:
constraints 1 to 5 hold or sorta-hold and constraint 6 reads "not yet",
so the next step is the auto-prototype in a fork or a disposable clone with the verification harness above (expect install 3 to relink after the fix).
`.out-of-scope/` was listed on 2026-10-29:
its entries (bun install,
cargo workspace,
claude-code upstream bugs,
codex harness,
jsr,
lightningcss,
low-impact typescript formatting,
module es monolith,
pi gpt55 long context,
terminal title fork parity tests,
typescript project references) do not name pnpm repeat-install behavior,
so no exemption applies,
though `bun-install.md` and `cargo-workspace.md` were not opened to rule out a broader class.
