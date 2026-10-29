# pnpm 11.28.9 and 12.16.1: a plain `pnpm install` prints `Already up to date` and leaves a missing declared workspace link missing

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

The fast path is `check_optimistic_repeat_install` in [`pnpm/crates/package-manager/src/optimistic_repeat_install.rs`][pnpm-fast-path-comment].
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
[`freshness.rs:257-265`][pnpm-freshness-checks] runs the two filesystem checks that "prove installed":

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
[`modules_dirs.rs:164-168`][pnpm-dangling-doc] (its documentation) and [`modules_dirs.rs:205-207`][pnpm-is-dangling] (`is_dangling_link`),
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

### The draft's own reproduction

The steps of the issue draft (an `app` project depending on `lib` through `workspace:*`) were run exactly as written on 2026-10-29,
in a throwaway workspace with its own `HOME`,
with the installed `pnpm` 12.16.1.
The first harness above used a different layout,
so this run is what backs the draft:

```text
pnpm 12.16.1
step 1: install exit 0 | app/node_modules/lib: symlink -> ../../lib | output: Scope: all 3 workspace projects / Already up to date / Done in 135ms using pnpm v12.16.1
step 2: after rm: absent
step 3: install exit 0 | link: absent | output: Scope: all 3 workspace projects / Already up to date / Done in 20ms using pnpm v12.16.1
step 4: install --force exit 0 | link: symlink -> ../../lib
```

### Version matrix

The reproduction repository's workflow runs the draft's steps on a clean `ubuntu-24.04` runner for each pnpm release,
each downloaded from its GitHub release and checked against a pinned SHA-256
(run [40911641893](https://github.com/Aquaticat/pnpm-missing-workspace-link-repro/actions/runs/40911641893),
2026-10-29):

- 12.16.1:
  the step named "Expected" fails,
  so the link stays missing.
- 12.16.1 with `PATH` limited to `/usr/bin:/bin` and the pnpm directory,
  so that no Node.js is on `PATH`:
  the same failure,
  so Node.js is not involved.
- 11.28.9:
  the same failure,
  so pnpm 11 is affected.
- 10.34.9:
  the job succeeds,
  so pnpm 10 recreates the link and is not affected.

The pnpm 11 and pnpm 10 source was not read;
those two results are behavior only.

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

## Prototype

A fix compatible with pnpm's architecture,
prototyped on a disposable clone of `https://github.com/pnpm/pnpm.git`
at commit `f7d7e6173ad68ee44918173bdded12602966cb80`
(origin and commit confirmed with `git remote get-url origin` and `git rev-parse HEAD` before editing).
The diff is [pnpm-missing-workspace-link-repeat-install-fast-path.patch](pnpm-missing-workspace-link-repeat-install-fast-path.patch)
(7 files,
120 insertions,
2 deletions).

What it changes:

- `modules_dirs.rs`:
  new `workspace_dependency_link_missing`,
  which reports a `workspace:` dependency with no entry at all in its project's modules directory.
  It applies only under the isolated linker without `dedupeDirectDeps`,
  and only to the non-optional groups the install includes,
  because skipped optional dependencies and deduplicated siblings have no entry by design.
- `freshness.rs`:
  a new `Stale::MissingWorkspaceLink`,
  checked next to the dangling-link check under the same gate policy,
  and only when the scope holds every project,
  because a filtered install leaves unselected projects as they are and could not fix an absent link there.
- `optimistic_repeat_install.rs` and `deps_status.rs`:
  the two `match` arms the new variant needs.
- `pnpm/crates/cli/tests/suite/repeat_install.rs`:
  `repeat_install_relinks_a_missing_workspace_dependency_link`,
  which installs a two-project workspace,
  deletes `app/node_modules/lib`,
  installs again and asserts the link is back.
- `optimistic_repeat_install/tests.rs` and `tests/injected.rs`:
  two fixtures that simulate an install now create the `node_modules` entry a real install makes for a `workspace:` dependency.
  Without that,
  five unit tests in `pnpm-package-manager` that assert "up to date" for a workspace dependency with no link failed with the fix,
  which is the buggy state.

How it was verified (`proto-run.ts` under `${HOME}/temp/agent/`,
one container,
no credentials,
10 GiB and 6 CPUs,
only the disposable clone mounted):

1. Stash the fix (`git stash push -- pnpm/crates/package-manager`) and run the new test.
2. Run the neighbouring selections with the fix stashed and record the failing test names:
   `repeat_install` (CLI suite,
   includes the oracle suites),
   `install_filters`,
   `workspace_install`,
   and `optimistic_repeat_install` in `pnpm-package-manager`.
3. Restore the fix and repeat both steps.
4. Compare the failing lists.

Recorded output of the last run:

```text
PRE-PATCH new-test exit=101
PRE repeat_install failing=190
PRE install_filters failing=3
PRE workspace_install failing=16
PRE package-manager failing=0
POST-PATCH new-test exit=0
POST repeat_install failing=189
POST install_filters failing=3
POST workspace_install failing=16
POST package-manager failing=0
-- repeat: only pre
test repeat_install::repeat_install_relinks_a_missing_workspace_dependency_link ... FAILED
```

Every other "only post" and "only pre" list was empty,
so no test fails only with the fix,
and the only difference is the new test going from failing to passing.

Limits of this evidence:

- This first run's container had no Node.js,
  so it failed about 190 CLI repeat-install tests,
  3 filter tests and 16 workspace tests in both arms,
  and a regression in one of those could not show up.
  The clean rerun below,
  with Node.js installed,
  supersedes this limit.
  The package-manager unit tests listed no failures in either arm
  (the last run recorded failing counts,
  not pass counts;
  an earlier run of the same selection reported 186 passed and the five failures the fixture change removed).
- An earlier run of the same prototype failed five package-manager unit tests;
  the fixture change above is what removed them,
  and that is the part a reviewer should read most closely.
- Not changed:
  after the fix a repeat install still prints `Already up to date` even though it has relinked the missing link
  (the pipeline runs,
  as the `Lockfile is up to date, resolution step is skipped` line shows),
  so the test asserts the link and not the message.
- Not built or run:
  the hoisted and `pnp` linkers,
  `dedupeDirectDeps` and optional dependencies,
  which the fix skips on purpose and which have no test of their own here.

### Clean rerun with Node.js (2026-10-29)

The first run's container had no Node.js,
which is why `pnpmfile` hooks and lifecycle scripts failed (`running updateConfig hook ... No such file or directory`).
The rerun used a fresh clone of current `main` (`cf2556e97c`),
the patch applied with `git apply` unchanged,
and `nodejs` and `npm` installed in the container
(CachyOS v4 image,
24 GiB,
12 CPUs,
only the disposable clone mounted,
run as a capped systemd user unit).
The same steps as above gave:

```text
PRE-PATCH new-test exit=101
PRE repeat_install failing=2
PRE install_filters failing=0
PRE workspace_install failing=3
PRE package-manager failing=0
POST-PATCH new-test exit=0
POST repeat_install failing=1
POST install_filters failing=0
POST workspace_install failing=3
POST package-manager failing=0
```

The failing lists in both arms:

- `hoisted_node_linker::repeat_install::an_interrupted_hoisted_install_forgets_the_locations_it_replaces`
  fails in both.
- `repeat_install::repeat_install_relinks_a_missing_workspace_dependency_link` fails only without the fix.
- `pnpr_install::workspace::filtered_workspace_install_via_pnpr_materializes_the_root_and_selected_closure`,
  `workspace_install::freshness::changed_registry_specifier_in_workspace_importer_invalidates_lockfile`
  and `workspace_install::optional_peer_is_not_supplied_from_the_lockfile_by_a_package_whose_peers_the_importer_rejects`
  fail in both.

Every "only post" list was empty,
so no test fails only with the fix.
The four tests that fail in both arms were not investigated;
they are identical with and without the change.

### pnpm 11 (TypeScript) prototype

pnpm 11 has the same exemption in `pnpm11/deps/status/src/findDanglingDirectDependencyLink.ts`
([doc comment][pnpm11-dangling-doc]):
"A missing entry is not a broken link:
skipped optional and excluded dependencies have none."
The prototype
([`pnpm-missing-workspace-link-repeat-install-fast-path.v11.patch`](pnpm-missing-workspace-link-repeat-install-fast-path.v11.patch),
the source file plus a new test file,
99 insertions and 9 deletions)
treats a missing entry as stale when it is a non-optional `workspace:` dependency,
the linker is isolated,
and `dedupeDirectDeps` is off.

Harness:
the original and patched file at commit `cf2556e97c`,
with the three workspace imports redirected to stubs,
run by `node` 24 in `podman run --network=none --memory=1g --cpus=1` against a disposable directory per case.
Result on 2026-10-29:

```text
original=fresh patched=fresh | workspace dependency, link present
original=fresh patched=stale | workspace dependency, link absent
original=fresh patched=stale | workspace devDependency, link absent
original=stale patched=stale | workspace dependency, link dangling
original=fresh patched=fresh | registry dependency, entry absent
original=fresh patched=fresh | workspace optionalDependency, entry absent
original=fresh patched=fresh | workspace dependency, hoisted linker, entry absent
original=fresh patched=fresh | workspace dependency, dedupeDirectDeps, entry absent
original=fresh patched=fresh | workspace devDependency excluded by --prod, entry absent
PATCHED MATCHES EXPECTATIONS
```

The two "link absent" rows are the positive control:
the original says fresh (the bug) and the patched file says stale (the fix).

Run in pnpm 11's own setup:
the same cases are a jest test file,
`pnpm11/deps/status/test/findDanglingDirectDependencyLink.test.ts`,
in the patch.
It ran on 2026-10-29 in the clean container of the Rust rerun (Node.js installed,
`tsgo --build` then `jest`,
on commit `cf2556e97c`):

```text
original source: Tests: 2 failed, 7 passed, 9 total
  failing: a missing entry for a workspace dependency in dependencies is stale
           a missing entry for a workspace dependency in devDependencies is stale
patched source:  Tests: 9 passed, 9 total
```

The two failures in the original arm are the positive control.

What makes a stale result relink the link:
`installDeps.ts` returns `false` from the repeat-install shortcut when `findDanglingDirectDependencyLink(opts) != null`,
so the full install runs
([`installDeps.ts:228-234`](https://github.com/pnpm/pnpm/blob/cf2556e97c3b344e63e9504cf2da3d96426f1748/pnpm11/installing/commands/src/installDeps.ts?plain=1#L228-L234)).
That is the path the existing dangling-link case already takes.

Limits:
a pnpm 11 CLI build was not run end to end,
and the new test and the source change were not run through pnpm 11's ESLint step.

## Upstream filing decision

Walk of the six constraints,
with the default policy of not filing.

1. Is it really upstream's fault?
   Partly.
   The behavior follows a documented choice ([`modules_dirs.rs:167-168`][pnpm-dangling-doc]:
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
   [`CONTRIBUTING.md`][pnpm-contributing-ai] (section "AI-assisted contributions",
   lines 325-345) welcomes agent-assisted contributions with a disclosure footer such as `Written by an agent (Claude Code, claude-opus-4-7).`,
   and warns that unvetted agent output,
   duplicate PRs and PRs that do not compile may be closed.
   No ban found.
5. Will they likely fix it?
   No signal either way.
   `gh search issues --repo pnpm/pnpm` for `missing workspace link Already up to date` and for `repeat install missing symlink node_modules not relinked` each returned no issues (checked 2026-10-29).
6. Have we prototyped a minimal fix compatible with their architecture?
   Yes,
   see "Prototype" above:
   a sibling check next to `direct_dependency_link_dangling` (`workspace_dependency_link_missing`),
   a new `Stale` variant,
   and a test.
   The new test fails without the fix and passes with it,
   and no other test in the four selections run differs between the two arms.
   The clean rerun with Node.js installed ran the CLI repeat-install,
   filter and workspace suites and the package-manager unit tests:
   the only difference between the arms is the new test.
   The pnpm 11 fix has its own jest test,
   failing on the original source and passing on the patched one.

Decision:
constraints 1 to 6 hold with the caveats written above,
so the draft below was fileable.
Filing is an outward-facing action on a third-party tracker,
so `AGENTS.md` requires the repository owner's go-ahead before anything is posted;
the owner chose "file the issue only" on 2026-10-29 and the issue was filed as
[pnpm/pnpm#17485](https://github.com/pnpm/pnpm/issues/17485).
The filed body follows pnpm's bug form field by field and links both patches from the reproduction repository
(commit `81b20eb412`,
directory `patches/`) instead of attaching them.
The issue type `Bug` that the form sets could not be set through the API by a non-maintainer,
so the type is empty;
the triage automation added the label `area: monorepo`.

Outcome,
read on 2026-10-29 from the issue timeline and the pull request:
the report was fixed upstream by someone else,
and no pull request from this account was needed.
About 40 minutes after the filing,
`snmsoodan` opened
[pnpm/pnpm#17487](https://github.com/pnpm/pnpm/pull/17487) with `Closes pnpm/pnpm#17485`,
and `zkochan` merged it at 21:19 UTC (merge commit `39bb7b93c5`,
review decision `APPROVED`),
which closed the issue.
Nobody commented on the issue and no maintainer asked for a pull request.
The merged change covers both stacks,
14 files:
the Rust v12 path (`pnpm/crates/package-manager/src/optimistic_repeat_install/modules_dirs.rs` and a new `modules_dirs/direct_links.rs`)
and the TypeScript v11 path (`pnpm11/deps/status/src/findDanglingDirectDependencyLink.ts`,
a jest test,
and a CLI test in `pnpm11/pnpm/test/install/misc.ts`),
plus a changeset.
It treats a missing entry as dangling only when the manifest declares a `workspace:` or `link:` spec,
no root entry stands in for it,
the lockfile links it,
the linker symlinks direct dependencies,
and the project is held by the install.
The pull request lists two known gaps where a deleted link is still not detected:
a pinned `lockfileDir` or custom virtual-store directory for the current-lockfile fallback,
and an override that retargets a workspace dependency while the root keeps a link to the declared target.
The patches in the reproduction repository and the two `.patch` files beside this doc are superseded and were never posted as pull requests.
Not checked:
whether a release containing the merge fixes the reproduction (the matrix above predates it),
and whether the merge reaches the pnpm 11 line (the pull request targets `main`).

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

## Upstream filing artifact

Draft of a new issue,
filed as pnpm/pnpm#17485 on 2026-10-29 (the block is the pre-filing draft;
the filed text rearranges it into the form's fields and replaces "attached" with links).
The duplicate search found nothing,
so this is a new-issue draft and not a comment.
The block below is the issue exactly as it would be posted:
it holds no notes to the filer,
and it carries the disclosure footer that pnpm's `CONTRIBUTING.md` asks for.

Before anyone posts it:

- Attach the patch files `pnpm-missing-workspace-link-repeat-install-fast-path.patch` and
  `pnpm-missing-workspace-link-repeat-install-fast-path.v11.patch` from this directory;
  the block says they are attached.
- Weigh the risk to the poster's account before posting anything.
  Some repositories run trust or blocklist systems (Vouch is one;
  others exist and this doc did not survey them) that close or lock issues and pull requests from unvouched users,
  and a maintainer can denounce a poster,
  which can follow that person across projects that share lists.
  Other people may see it,
  interviewers and employers among them.
  An agent-written report that a maintainer judges low-effort is the case these systems exist for.
  At the commit pinned above,
  pnpm's `.github`,
  `AGENTS.md` and `CONTRIBUTING.md` mention neither `vouch` nor `denounce`;
  check the live repository and its contribution policy again on the day of posting.
  Filing is the poster's decision.
  Readiness check of 2026-10-29,
  run after the clean rerun:
  ready,
  and the recommendation is to file the issue below (see the ranking in the next bullet).
  Every item below was run against the live repositories that day:
  - Releases:
    the latest are 12.16.1,
    11.28.9 and 10.34.9,
    the versions tested.
  - Duplicates:
    issue searches for the symptom words returned nothing,
    and a positive-control search (`Already up to date`) returned five issues.
    Three closed ones were read:
    #12090 and #12748 report a fresh clone with no `node_modules` at all (and `--force` does not fix #12748),
    and #17468 reports a `readPackage` hook that stops the fast path from ever applying.
    None covers one removed link with unchanged state,
    which `--force` repairs.
  - Prior art:
    #15569 states the exemption this report asks to narrow,
    and the draft cites it.
  - Trust lists:
    no `VOUCHED.td`,
    `.github/VOUCHED.td`,
    `vouch.td` or `.github/vouch.td` exists on the default branch,
    and no workflow name mentions vouch,
    denounce or trust.
  - The code:
    the Rust patch applies unchanged to `main` at `cf2556e97c`,
    so the check is still there;
    the pnpm 11 file was read at the same commit.
  - Versions:
    11.28.9 and 12.16.1 reproduce and 10.34.9 does not,
    on clean runners.
  - Fix evidence:
    Rust integration test fails without the fix and passes with it,
    with an identical failing list otherwise;
    pnpm 11 jest test does the same.
  - Intake:
    the form fields are mapped below,
    and the Node.js field is answered by the run with no Node.js on `PATH`.
  Caveats that do not block the recommendation:
  the pnpm 11 change was not run through pnpm 11's ESLint step or a CLI build;
  four tests fail identically with and without the change and were not investigated;
  and the live checks should be repeated if posting waits more than a day.
  The recommendation does not remove the risk to the poster above;
  it only says the report is ready.
- Fill pnpm's bug form field by field;
  its blank-issue option is off and its contact links say issues are for bug reports only
  (read from `.github/ISSUE_TEMPLATE/bug-report.yaml` and `config.yml` on the default branch on 2026-10-29).
  The block below maps to the form like this:
  the reproduction repository URL goes in "Link to the code that reproduces this issue";
  the numbered steps go in "Reproduction steps";
  the paragraphs before the steps go in "Describe the Bug";
  the sentence under "Expected" goes in "Expected Behavior".
  The Node.js version is a required field.
  The matrix run with no Node.js on `PATH` still reproduces,
  so the answer is that Node.js is not used,
  with that run as the evidence;
  the poster confirms it before posting.
  "I verified that the issue exists in the latest pnpm release" was true for 12.16.1 on 2026-10-29,
  the latest release then,
  so re-check it on the day.
  "I am working on a fix" assigns the issue to the poster;
  tick it only if a pull request is planned.
- Know what happens after posting.
  pnpm's `triage-issues.yml` runs an automated agent on every issue from a human account
  that labels it and posts a triage comment,
  and `issue-fix-intent.sh` comments to ask whether the author plans a pull request when the form's fix box is absent
  (read on 2026-10-29).
- Know the limits of the evidence.
  The behavior was tested on pnpm 12.16.1 and 11.28.9 (affected) and 10.34.9 (not affected).
  pnpm's contribution checklist says bug fixes go into every affected version,
  and the prototype patches cover both stacks:
  the Rust code (pnpm 12) with an integration test,
  and the TypeScript function (pnpm 11) with a jest test.
- Issue or pull request.
  Ranking:
  issue first,
  pull request second.
  The issue ranks first because #15569 states the exemption on purpose,
  so the maintainers should decide whether to narrow it before reviewing code,
  and a pull request that overturns a stated decision is the likelier to be closed.
  The pull request ranks second because both patches already have tests and a clean baseline,
  so it is ready if the maintainers ask for it,
  and it must then follow `.github/pull_request_template.md` (Summary,
  Squash Commit Body,
  Checklist).
  `CONTRIBUTING.md` warns that unvetted agent output can be closed without review.

~~~md
Title: `pnpm install` reports "Already up to date" and leaves a declared `workspace:` dependency link missing (11.28.9, 12.16.1)

When manifests, lockfile and settings still match the digests in `node_modules/.pnpm-workspace-state-v1.json`,
the repeat-install fast path returns before linking.
Its check on individual links, `direct_dependency_link_dangling` in
[`pnpm/crates/package-manager/src/optimistic_repeat_install/modules_dirs.rs`](https://github.com/pnpm/pnpm/blob/f7d7e6173ad68ee44918173bdded12602966cb80/pnpm/crates/package-manager/src/optimistic_repeat_install/modules_dirs.rs#L164-L168),
treats a link that exists but points nowhere as stale and an absent entry as healthy
([`is_dangling_link`](https://github.com/pnpm/pnpm/blob/f7d7e6173ad68ee44918173bdded12602966cb80/pnpm/crates/package-manager/src/optimistic_repeat_install/modules_dirs.rs#L205-L207): `fs::metadata(path).is_err() && fs::symlink_metadata(path).is_ok()`).
The doc comment says a missing entry is not a broken link because skipped optional and excluded dependencies have none,
but a non-optional `workspace:` dependency under the isolated linker has no such reason.

Prior art:
[pnpm/pnpm#15569](https://github.com/pnpm/pnpm/pull/15569) added the check on links that exist but point nowhere,
closing [#9758](https://github.com/pnpm/pnpm/issues/9758),
and its description states the exemption:
"[A missing entry still passes, because skipped optional and excluded dependencies have none.](https://github.com/pnpm/pnpm/pull/15569#:~:text=A%20missing%20entry%20still%20passes%2C%20because%20skipped%20optional%20and%20excluded%20dependencies%20have%20none.)"
This report asks to narrow that exemption for non-optional `workspace:` dependencies,
which are always linked by the isolated linker and have no such reason to be absent.

Reproduction (pnpm 12.16.1 and 11.28.9, default linker, no registry access needed; pnpm 10.34.9 recreates the link):

```text
pnpm-workspace.yaml:   packages:
                         - app
                         - lib
package.json:          {"name":"ws-root","private":true,"version":"1.0.0"}
lib/package.json:      {"name":"lib","version":"1.0.0"}
app/package.json:      {"name":"app","version":"1.0.0","dependencies":{"lib":"workspace:*"}}
```

1. `pnpm install --offline`; `app/node_modules/lib` exists, a symlink to `../../lib` (this install also ends with `Already up to date`).
2. `rm app/node_modules/lib`.
3. `pnpm install --offline` prints `Already up to date`, exits 0 and leaves the link absent.
4. `pnpm install --offline --force` restores it.

Expected: step 3 recreates `app/node_modules/lib`, as step 4 does.

The same steps run as a GitHub Actions workflow in
https://github.com/Aquaticat/pnpm-missing-workspace-link-repro
(each pnpm release installed from its GitHub release and checked against a pinned SHA-256).
A red job means the bug reproduced:
the step named "Expected" fails with `ls: cannot access 'app/node_modules/lib': No such file or directory`.
[Run 40911641893](https://github.com/Aquaticat/pnpm-missing-workspace-link-repro/actions/runs/40911641893):
12.16.1 and 11.28.9 fail at that step, 10.34.9 passes,
and 12.16.1 still fails with no Node.js on `PATH`, so Node.js is not involved.

Suggested fix: a sibling check beside the dangling-link check that reports a non-optional `workspace:` dependency with no
entry in its project's modules directory, limited to the isolated linker without `dedupeDirectDeps`, and only when the
install's scope holds every project.
Prototype patches are attached for both stacks.
The Rust one (pnpm 12) has 7 files, 120 insertions and 2 deletions.
Its new integration test, `repeat_install_relinks_a_missing_workspace_dependency_link`, fails without the change and passes with it,
and no other test in the four selections compared (`repeat_install`, `install_filters` and `workspace_install` in `pnpm-cli`,
and `optimistic_repeat_install` in `pnpm-package-manager`) fails only with the change.
The failing lists are otherwise identical: four tests fail with and without the change on current `main` (cf2556e).
pnpm 11 has the same exemption in the doc comment of
[`findDanglingDirectDependencyLink.ts`](https://github.com/pnpm/pnpm/blob/cf2556e97c3b344e63e9504cf2da3d96426f1748/pnpm11/deps/status/src/findDanglingDirectDependencyLink.ts?plain=1#L14-L23).
The pnpm 11 patch changes that file and adds a jest test beside the existing ones.
On the original source 2 of its 9 cases fail (a missing entry for a `workspace:` dependency in `dependencies` and in `devDependencies`),
and on the patched source all 9 pass.
Known limits: with the change the install pipeline runs and the link is restored (the new Rust test asserts the link),
but the output still ends with `Already up to date`;
that message also ends the first install above, which does create the link, so it is not specific to this case.
The pnpm 11 change was not built into a CLI or run through its ESLint step.

Written by an agent (Claude Code, claude-sonnet-5-5).
~~~

[pnpm11-dangling-doc]: https://github.com/pnpm/pnpm/blob/cf2556e97c3b344e63e9504cf2da3d96426f1748/pnpm11/deps/status/src/findDanglingDirectDependencyLink.ts?plain=1#L14-L23
[pnpm-fast-path-comment]: https://github.com/pnpm/pnpm/blob/f7d7e6173ad68ee44918173bdded12602966cb80/pnpm/crates/package-manager/src/optimistic_repeat_install.rs#L4-L9
[pnpm-freshness-checks]: https://github.com/pnpm/pnpm/blob/f7d7e6173ad68ee44918173bdded12602966cb80/pnpm/crates/package-manager/src/optimistic_repeat_install/freshness.rs#L257-L265
[pnpm-dangling-doc]: https://github.com/pnpm/pnpm/blob/f7d7e6173ad68ee44918173bdded12602966cb80/pnpm/crates/package-manager/src/optimistic_repeat_install/modules_dirs.rs#L164-L168
[pnpm-is-dangling]: https://github.com/pnpm/pnpm/blob/f7d7e6173ad68ee44918173bdded12602966cb80/pnpm/crates/package-manager/src/optimistic_repeat_install/modules_dirs.rs#L205-L207
[pnpm-contributing-ai]: https://github.com/pnpm/pnpm/blob/f7d7e6173ad68ee44918173bdded12602966cb80/CONTRIBUTING.md?plain=1#L325-L345
