# Build and release lifecycle

## Retired native pipeline, 2026-10-19

The user removed `bin`, `native`, and the generated `dist` tree.
The agent introduced this copied/transformed Pi architecture without user approval.
The rebuild must use Pi's public plugin API and must not modify Pi source or runtime internals.
Ordinary plugin output may use `dist`; the native Pi build must not return.
Historical source paths in this document now refer to the preserved pre-rebuild worktree
at `~/temp/agent/pi-auto-mode-before-rebuild-20261019/package/pi-plugin/auto-mode/`.
The historical claim that the extension interface cannot supply the required behavior is not established
for the new public-plugin design and is withdrawn.

The retained lifecycle measurements describe the retired implementation only.

## Purpose and interface

`native/build.ts` builds a fresh native release and publishes the stable `dist/native` command target.
`native/build-input.ts` validates the qualified upstream source and installed package version.
`native/build-emission.ts` emits upstream packages and links their installed dependencies.
`native/contract/integration/native-batch/prepare.ts` applies owned native transformations.
`native/build-activation.ts` publishes the verified release through atomic link replacement.

The native command depends on this module because the host needs complete-group execution seams
that the installed third-party extension interface does not supply.
The package transforms its own qualified source copy rather than editing installed upstream files.

Upstream TypeScript remains in the verified checkout used by the compiler.
`native/build-emission.ts` compiles from that checkout directly into the fresh release's `dist` directories
and copies non-TypeScript assets from the same source.
The emitted package metadata and qualification manifest remain with the runtime.
This removes the second source tree from each release without changing the compiler or native transformations.
Release `native-release-FcmP01` contains 1,031 regular files totaling 8,225,739 bytes,
compared with 1,499 files totaling 12,743,268 bytes in `native-release-J4fNNL`.
All 507 qualified SDK runtime modules and assets remain byte-identical across the releases.
The 18-mode native lifecycle catalog and disposable publication/rollback test pass;
the package type check also passes.

## Invariants and retained releases

The source revision and installed upstream version must match the build's qualification.
Native transformations reject unexpected source structure.
Generated imports bind to one release's maintained policy implementation.
Publication occurs after verification.

Earlier release paths remain stable for running sessions.
Deleting a previous release merely because the active link changed could break a later dynamic import
in a session that still owns the old release.
The initial footprint contains repeated native releases because each build retains these immutable paths.
This is an identified storage cost,
not proof that every retained release remains in use.

The `--owned` build reuses the already emitted qualified Pi graph,
then regenerates owned transformations.
`native/build-reuse.ts` shares unchanged files with the previous release after CLI verification
and before publication.
It pins the previous release once,
compares device,
mode,
size,
and complete SHA-256 contents,
then atomically replaces only the new release's matching file with a hard link.
Matching device identifiers do not guarantee that Linux permits a hard link:
distinct mounts of the same filesystem can still return `EXDEV`.
The build retains the complete independently verified new file in that case,
records the skipped optimization and continues publication.
Other link errors still propagate.

`native/build-reuse.platform.test.ts` exercises actual distinct mounts with equal device identifiers.
Its direct-link control first proves that the fixture returns `EXDEV`.
The actual native build command must then publish intact old and new bytes,
report zero reused files and leave no temporary link entries.
A subsequent real build on the same mount checks successful inode sharing.
The named task is `test:native:build-reuse:mount`,
with previous-file and output-directory arguments supplied by a disposable bounded mount fixture.
Frozen `532c20e618` first reproduces the defect through the helper;
the maintained consumer regression invokes the actual build command.
Changed files retain independent inodes.
The first recursive enumeration incorrectly assumed that Node would ignore directory symlinks.
On Node `26.12.0`,
recursive `readdir` follows them even with `withFileTypes: true`.
A committed real-build regression first proves `EACCES` for a disposable unreadable target,
then reproduces the build's attempt to scan that target through a copied contract symlink.
The replacement reads nonrecursive directory entries,
extends a sequential queue only for actual directories and compares only regular files.
The final sequential-queue consumer publishes successfully,
retains the opaque link and shares both root and nested files on one mount.
Fresh package types pass,
and scoped lint checks both maintained files with zero warnings and errors across 486 rules.
The source and bounded probe are retained in the
[Node directory traversal diagnosis](../../../../doc/troubleshooting/node-recursive-directory-symlinks.md).
It does not rewrite an old release.

Each actual build sends stdout and stderr directly to distinct per-invocation files before starting.
Failed children retain their complete output and final newline bytes.
Mutation verification proves both the `EXDEV` fallback and successful same-mount sharing:
removing either behavior fails the actual-command regression,
and restoring the implementation passes.
The rebuilt borrowed-directory mutant restores recursive enumeration and fails the third actual build with `EACCES`.
A separate mutant skips real child directories and fails the nested-file inode assertion after the second build.
Restoring the original implementation passes all three actual builds,
with six independent raw stdout/stderr files retained.
The private receipt is `reuse-traversal-built-mutants.json`
under `${HOME}/temp/agent/pi-mimo-preparation-failure-20261018`.

The supervisor's original log collector repeats the same traversal assumption outside the bounded child.
Its recovery reads the completed child receipt first,
restores the private mutation and follows only known fixture/log directories.
The user explicitly keeps root `RXI` unchanged.
The concrete lesson is that bounds and ownership must also cover diagnostic traversal and log collection.
The 2026-10-17 checkpoint `native-release-qWSxFz` shares 1,000 files totaling 7,518,255 bytes.
It contains 1,030 regular files totaling 8,211,396 bytes.
The emitted graph records five computed import sites and finds no missing static targets.
Its 794 conservatively reachable files total 6,099,319 bytes.
Contract fixtures and generation inputs remain because native tasks load them directly;
literal graph absence does not establish that they can be deleted.

Cleanup dry-run lists ignored cache,
dist,
and node_modules trees.
Root `HEAD`,
`config`,
`hooks`,
`objects`,
and `refs` do not exist despite matching ignore patterns.
Live Node/Pi process inspection and a complete repository package-consumer scan precede maintenance.
Maintenance removes 43 inactive releases and retains active `native-release-qWSxFz`
plus verified `native-release-mHAoem`.
Dependency links and source are preserved.
This is reviewed maintenance rather than automatic eviction of a running session's output.

## Process lifetime evidence

The 2026-10-18 cleanup audit exposes behaviors that invalidate a command-name-only process search.
The bounded replay mounts Node as `/fixture-node`.
Pi's qualified `dist/native/pi/coding-agent/dist/cli/setup.js` assigns `process.title` from `APP_NAME`,
which can replace the original module arguments visible through Linux `/proc/<pid>/cmdline`.
The repository is also accessible through paths that resolve to the same directory.
A search restricted to executable basenames or one literal repository path can therefore miss a running consumer.

The corrected audit examines all readable process command lines,
canonical and configured repository paths,
and the qualified package's configured application title and RPC title.
Executable,
working directory,
release references,
and disposable-home inode identify matched processes without printing their complete arguments.
The positive control finds both running configured-host replays under their `/fixture-node` aliases.
An empty result is usable cleanup evidence only after that positive control and after the replays terminate.
Unreadable processes and consumers whose release cannot be established remain explicit uncertainty;
do not delete their possible release while they run.

## Alternatives and verification

### Native CLI receipt fixture

The CLI network guard appends a private JSONL receipt before returning each local classification response.
It preserves original model and question names and rejects other routes or credentials.
The actual CLI consumer waits for child close,
requires nonempty classification evidence and checks complete captured stderr.
The receipt consumer independently uses standard JSONL parsing and checks the final LF byte,
so matching writer/reader escape mistakes cannot make the test pass.
Its write-failure control rejects a response when the receipt cannot be written.
A private rebuilt variant without the awaited write fails that maintained test.

The network unit's new missing-path case initially depends on its parent environment.
With `nano-spawn` `2.1.0`,
an omitted override inherits the parent's value.
The clean-parent run passes while an inherited nonempty receipt path reaches the forbidden probe.
This is an owned fixture defect exposed by supported process behavior;
the setup guard itself correctly rejects genuinely missing and empty paths.
Commit `a6417c7303` explicitly masks the inherited value before applying each case's overrides.
Independent frozen-source runs pass with both clean and contaminated parent environments.
The explicit empty-string case remains separate.
The source trace and reproduction are retained in
[`nano-spawn` environment inheritance](../../../../doc/troubleshooting/nano-spawn-environment-inheritance.md).

Overwriting the active directory would invalidate running module identities.
Editing installed upstream output would detach source qualification from deployment.
Rewriting a file shared with an earlier release could corrupt an active session.
Generation completes in a fresh release before identical files are shared;
later builds do not rewrite those paths.
Any deduplication must preserve independent immutable module paths and generated import identity.
Sequential file comparison bounds hashing and file-descriptor use.
Missing previous files and cross-mount hard-link refusal retain independent output;
other filesystem failures stop publication.

`build:native`,
`build:native:owned`,
`test:native:build-activation`,
and `test:native:build-imports` exercise generation,
publication rollback,
and import rewriting.
The installed command replay verifies the published consumer.
`native/build.unit.test.ts` publishes disposable successive releases,
checks an identical license file shares its inode,
checks changed CLI files remain independent,
and invokes the previous command after publication.
Its inode assertion failed before reuse was implemented,
then passes with the full package units.

Task names use `mise run //package/pi-plugin/auto-mode:<task>`.

## Scoped lint argument boundaries

The package's `lint:oxlint` and `format:oxlint` tasks accept a separate shell argument for each source path.
The actual worker initially quotes both paths together;
Mise and the repository's `parse_usage_args` helper correctly preserve one filename containing a space.
Oxlint then reports `No files found to lint`,
checks zero files and exits unsuccessfully.
That result supplies no clean-lint evidence.

The corrected worker invocation uses an explicit package cwd and separate absolute path arguments.
Its first lint checks both actual files,
and its formatter also reports two checked files.
A private probe executes the repository's actual argument-parser source through Mise
and confirms the single-argument and separate-argument cases.
The correction changes the invocation;
it does not change lint configuration or ignore patterns.
