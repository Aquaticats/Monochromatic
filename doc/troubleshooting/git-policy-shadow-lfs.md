# Scoped RPC commit fails at an unrelated LFS filter

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Note 2026-11-09T16:46:15-05:00:
these citations in this record name paths that no longer exist.
`package/pi-plugin/auto-mode/native/build-reuse.ts` left the tree on 2026-10-19 in `ff55ced353`.
`package/pi-plugin/auto-mode/native/build-reuse.platform.test.ts` left the tree on 2026-10-19 in `ff55ced353`.
`package/pi-plugin/auto-mode/native/` is not in the tree;
its files were deleted over several commits,
most of them on 2026-10-19 in `ff55ced353`.
`package/git-policy/cli/src/native/shadow_create.rs` is now
`package/git-policy/cli/src/native/shadow/shadow_create.rs`,
moved 2026-11-03 in `373765289c`.

Note 2026-11-10T13:20:00-05:00:
the read-only-LFS section of `git-policy-wrapper-launcher-path.md`
moved into this record,
because it reports the same failure as the 2026-10-18 consumer run recorded here
rather than a launcher-path finding.
Its probe established why the LFS temporary path is unavailable,
which the original consumer evidence had left open.

## Symptom

On 2026-10-18,
the configured MiMo RPC worker `pi-mimo-rpc-session-9Tl0ts` attempts a protected,
explicit-path commit of `package/pi-plugin/auto-mode/native/build-reuse.ts`
and `package/pi-plugin/auto-mode/native/build-reuse.platform.test.ts`.
The bounded Fedora 44 consumer has a disposable home,
2 GiB memory and two CPUs.
Its qualified wrapper is active.
The commit exits 128:

```text
Error cleaning Git LFS object: open <worktree>/.git/cli-git/shadow/<transaction>/lfs/tmp/2882421905: no such file or directory
error: packet write failed: Broken pipe
error: external filter 'git-lfs filter-process' failed
fatal: package/ide/app/design/screenshots/2026-10-05-annotations-as-built/caret-card-dark.png: clean filter 'lfs' failed
```

The paths this record names under `package/pi-plugin/auto-mode/native/` were removed on 2026-10-19
in `ff55ced353` (refactor(pi-plugin-auto-mode):
retire unapproved native Pi build);
no file in the
working tree carries them now.

The named LFS asset is outside the requested commit paths.
The owned source changes remain staged.
The worker stops Git retries and preserves its complete original tool event and stderr.
Supervisor delivery of those exact paths through the qualified host wrapper succeeds as `9363d8998b`.
This establishes an environment-sensitive boundary failure,
without identifying why the LFS temporary path is unavailable.
A second probe,
recorded under "Root cause" and "Verification" in this file,
identified the mechanism and showed raw Git failing the same way;
it did not inspect the consumer's mount permissions.

## Root cause

Git LFS creates its temporary files under `<git storage directory>/lfs/tmp`
(`git lfs env` prints it as `TempDir`) and offers no setting that moves only that directory;
`lfs.storage` moves the whole LFS store.
The wrapper's shadow repository links its `lfs` entry to the real `.git/lfs`
(`package/git-policy/cli/src/native/shadow/shadow_links.rs`,
moved there on 2026-11-03 in `373765289c` from
`package/git-policy/cli/src/native/shadow_links.rs`),
so the failing path is the shadow path
while the directory that must exist and be writable is the real one.
When `.git/lfs/tmp` is absent and `.git/lfs` cannot be written to,
Git LFS cannot create it and reports `no such file or directory` on the temporary file.
In the failing probes the clean filter ran for the unrelated modified asset,
not for the named files.

## Verification

The probes used disposable repositories with an LFS-tracked PNG modified in the worktree and an explicit ordinary-file
commit pathspec,
with the installed `0.1.6` wrapper and Git `2.56.0`.
A pathspec-scoped commit of ordinary source files through the wrapper then exits 128 while Git processes the
unrelated tracked LFS asset,
with Git LFS naming the missing temporary file:

```text
Error cleaning Git LFS object: open <worktree>/.git/cli-git/shadow/<transaction>/lfs/tmp/<number>: no such file or directory
error: external filter 'git-lfs filter-process' failed
fatal: <asset>: clean filter 'lfs' failed
```

The owned files stay staged and uncommitted,
and the same commit through the host wrapper succeeds.

- A writable `.git/lfs`,
  with or without `tmp`:
  the wrapper commits only the named file.
  A Fedora 44 container limited to 2 GiB and 2 CPUs,
  with a disposable `HOME` and no network,
  behaves the same.
- `.git/lfs` read-only and `tmp` absent:
  the wrapper fails with the message above,
  and raw Git fails with the same
  message naming `.git/lfs/tmp`,
  so the wrapper does not cause it.
- `.git/lfs` read-only and `tmp` present and writable:
  the wrapper commits only the named file.

The consumer's mount permissions were not inspected,
so that the consumer's `.git/lfs` was read-only is consistent
with the evidence but not confirmed.

## Owned boundary and report

`package/git-policy/cli/src/native/shadow_create.rs` creates the transaction's repository.
`shadow_config.rs` includes the real common configuration.
`commit_preparation.rs` supplies the shadow Git directory,
original worktree and private index to native commit.
The complete child environment,
filter process ownership and shadow lifetime still require controlled reproduction.

[Issue 1770](https://github.com/Aquaticats/Monochromatic/issues/1770) records the failed real consumer,
successful host delivery and required disposable native-Git/wrapper/filter controls.
Searches across all issue states for `LFS shadow`,
`filter-process` and `unable to write new index file` find no matching incident before filing.
The retained worker event is `call_f94fb711ff224917b3b6ec0b` in
`${HOME}/temp/agent/pi-mimo-rpc-session-9Tl0ts/host-events.jsonl`.

No wrapper repair is claimed.
No Git,
Git LFS,
Pi,
Node or Linux defect is attributed from this observation.
The separate source worker's `unable to write new index file` failure
and the historical `bad line length character: Not ` failure remain separate incidents until their boundaries match.
This also differs from the missing-wrapper PATH incident in
[issue 1768](https://github.com/Aquaticats/Monochromatic/issues/1768)
and [divergent auto-push reconciliation](git-policy-auto-push-reconciliation.md).

## Remedy and ownership

The environment that runs the child (container mounts or ownership) must leave `.git/lfs` writable,
or create `.git/lfs/tmp` with write access before the child starts.
Launch acceptance should run a scoped commit of an ordinary file in a repository that tracks LFS files.
The wrapper cannot supply the directory:
it runs with the child's permissions,
and redirecting storage with `lfs.storage` would discard LFS objects of an owned asset when the shadow is removed.
