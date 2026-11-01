# Main merge into translation repair

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/git-executable` is now `@monochromatic-dev/git-executable-core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/toml-edit.fuzz` is now `package/toml-edit/core.fuzz`;
> `@monochromatic-dev/module-fs-path` is now `@monochromatic-dev/fs-path-core`.

## Requested operation

The user requests merging the main worktree into the repair worktree
and delegates conflict resolution.
The selected main commit is `73120ef24c810db3b884445aad38f934490f9904`.
The repair branch starts at `ea3087d9c477b133df2b15bcaa401987d42bd78a`.
Main is clean at selection and the subsequent check.
The target branch is `translation-repair-rebased`.

The first merge refuses to overwrite the uncommitted `mise.lock`.
The user explicitly authorizes temporarily preserving it and reconciling conflicts.
Its original SHA-256 is
`1059ba85f2498ac4d67f51bf11292d86b66c6f037082076814ad5bb7b1939432`.
Private backup `main-merge-lockfile-VyH4kO` under agent scratch contains exact bytes and the original patch.
Stash `7a55f12e9d76bfd9998f07cda9512180fb79e9aa` contains the same bytes
and has the repair starting commit as its first parent.
Only `mise.lock` is stashed.
Restoration is performed after the merge commit,
so the local changes do not enter that commit.

## Verified merge scope

The shared Git resolver reconciliation and post-merge repair lint work are complete
within the recorded checks.
The merge retains the explicit dependency-overlay and inherited Git CLI lint limitations;
it does not qualify the complete repository,
comparison logging integration or publication.
The final real subpath lookup fails because main commit `2aef2d409`
removed `package/git-policy/cli/src/resolve-git.ts`,
while automatic merging retained its barrel export and repair imports.
The declared-export-name inventory did not prove that target existed.
The selected package matrices remain valid within their recorded scope,
but they did not compile the Git CLI or translation-repair.

[Issue 517](https://github.com/Aquaticat/Monochromatic/issues/517) records this separate merge defect
and explicitly states that the handling has not been reviewed.
Commit `52f3e1a00` migrates the repair imports to `@monochromatic-dev/git-executable/ts`,
removes the obsolete CLI export and wildcard subpath,
and changes the repair package dependency to the shared owner.
The root builder also captures working directory and Windows installation-root inputs
before caller getters or logging can change lookup context.
One missing declared workspace link is added without replacing existing links.
That is not a full dependency installation.

Native lock regeneration changes only the repair importer's dependency name and relative link.
`devtypes-U8tPOX` passes translation-repair's type check
with no source changes or OOM events.
`devbuild-nRe1Ob` passes the normal repair build,
and the preparation bootstrap also rebuilds successfully.
The first affected-consumer run,
`devtest-s4K1Wy`,
fails two legacy observers that expect whole-file `readFile` calls;
the shared resolver inspects through file handles instead.
The sampler and root-input tests now observe `open`,
prove their counters with a positive control,
and change PATH after first inspection so successful resolver caching cannot hide lost pin ownership.
The R2 sequence passes fresh types,
normal build,
bootstrap and all selected affected-consumer unit files.
Its final unit root is `devtest-j4I93h`.
`merge-resolver-guards-9R6KaU` then proves both updated controls:
each pin-removal mutant builds and triggers its designated ordinary assertion,
followed by a fresh restored build and both passing tests.
Real source and dist remain unchanged.
The first proof attempt uses the wrong assertion wording and is not counted.

The shared Git owner passes build,
types,
complete units and read-only lint in `main-merge-package-checks-ckDzPg`.
The Git CLI builds and type-checks,
and its selected source-level resolver tests pass.
Its full lint remains red with 83 artifact-import findings across 37 tests,
all byte-identical to selected main.
[Issue 518](https://github.com/Aquaticat/Monochromatic/issues/518) tracks that inherited package-completeness defect;
no suppression or broad Git CLI test/API migration is applied as part of the resolver reconciliation.
The initial owner lint attempt stops before diagnostics because the harness omitted the generated config output mount.

`main-merge-final-audit-DobZLC` verifies generated-output parity,
merge/stash/index invariants and all tracked marker forms,
including diff3 markers,
with a disposable positive control.
The regenerated lock passes the owned-cache frozen check in `main-merge-lock-generation-bAjmaG`.
The user explicitly relaxes the former 2 GiB container ceiling for this 64 GB host.
After checking current available memory,
`devlint256go1-88qhDp` uses 6 GiB RAM and completes without an OOM kill.
It reports 90375 inherited TSDoc-prefix warnings and seven layout warnings.
Private formatter `devformat-YKHlPN` completes with zero warnings and errors,
using `2204835840` peak memory bytes with no OOM events.
`main-merge-format-review-570dgD` checks all 1472 changed files:
normalized executable ASTs match,
including preserved template raw strings.
Comment changes are prefix removals except two exact duplicated-prefix repairs
that produce recognized `@param l - stage logger` tags in `translate-repair.ts`.
The declaration parameters and parsed documentation are checked for both repairs.
All other comment text and ordering remain intact.

Commit `f3858b514` transfers the exact verified bytes,
with regular-file,
mode,
path-set and pre/post-hash checks.
These changes remain explicitly unreviewed in issue #514.
The post-transfer sequence passes fresh types,
normal/bootstrap builds and affected units (`devtest-r2T4vU`).
Positive control `devlint256go1control-xuqvsT` reports exactly the missing-parameter warning
and unhandled-promise error introduced in its private source copy.
Restored `devlint256go1-1a4Xnt` reports zero warnings and zero errors
across 1561 files and 485 rules,
with `2171211776` peak bytes and no OOM events.
No real source is changed by either lint control.
BOX now requires host-sized bounds,
and `CLAUDE.md` is regenerated through the canonical owner.
Historical 2 GiB evidence is not relabeled as a 6 GiB run.

Task61's bounded lint and style verification is complete.
Final merge Markdown verification passes for the modified guidance and troubleshooting documents.
The retained cleanup record removes only the exact owned stopped IDs described in `Evidence retention and cleanup`.

Proposed instruction for merge verification:
check export targets and compile their affected consumers,
not only the union of declared names.
The target-resolution check is implemented in the private audit;
no additional `AGENTS.md` rule is applied yet.

## Conflict resolutions

The merge produces 14 conflicting paths.
Their original stages and working preimages are retained in private scratch
`main-merge-conflicts-EovoyE/conflicts.json`.

- `AGENTS.md` keeps both sets of additive rules:
  QDF/QPW/QNX,
  the visual-review and option rules,
  IMX/HLT and 1CB.
  The initial resolved file has 288 unique rule tags;
  the later standing issue-reporting instruction adds XIR.
- `CLAUDE.md` is generated through the actual file-enforcer CLI/library,
  using the canonical overwrite expression in `file-enforcer.config.ts`.
  Only its destination and staleness-manifest paths are redirected to a private candidate.
  The generated bytes equal main's unchanged generated prefix plus resolved `AGENTS.md`.
  Those exact bytes are transferred without manual editing.
- `doc/troubleshooting/README.md` retains both sets of links.
- `package/cli/markdown-lint/rolldown.node.config.ts` keeps one `perEntryNodeConfig` import.
- The Markdown rule barrel retains individual built-artifact test exports and main's rule collection.
- The pipe-table test retains the renderer-signature-derived `RenderableTable` type.
- The semantic-line-break rule retains the tag-like paragraph protection from `66345a092`,
  including its JSX component rationale,
  with main's comment layout.
- The async-time barrel retains `settleWithin` and its deadline/timer guidance.
- The logger barrel retains both the callback-observation API and main's internal test seams.
- The test barrel retains `caught`,
  `AssertionSite` and the assertion/error-format exports,
  removing duplicate export statements while preserving the internal-seam documentation.
- TOML escaping retains the raw-NUL documentation correction from `166735602`.
  Main's `abe8b9434` moves the fuzz campaign to `package/module/toml-edit.fuzz`;
  the correction follows `escape.ts` there and the retired runtime `src/fuzz/escape.ts` is removed.
- `pnpm-workspace.yaml` retains `pinyin-pro` and main's newer PostCSS floor.
- `pnpm-lock.yaml` is regenerated by the owning `pnpm` executable,
  not edited hunk by hunk.
  Main's complete lockfile is the regeneration input.

## Dependency and policy checks

The merged `cli-git.config.ts` adds the repository Markdown LFS-link policy.
The existing recursive-trust intent is unchanged.
Git-policy refuses staging until the configuration is reviewed and re-trusted.
Its first trust build rejects an unresolved static `ignore` import.
Node resolution confirms `ignore` is missing from the repair package
while main resolves its installed `ignore@7.0.8`.
The declared dependency link is created without replacing existing entries.
The subsequent explicit trust succeeds and retains a 79221-byte bundle.

The Markdown policy and file-enforcer CLI also require the newly added workspace dependency
`@monochromatic-dev/module-fs-path`.
Declared links for their respective package directories now point to this worktree's source owner.
No frozen runtime is replaced and no dependency package is patched.

The canonical file-enforcer candidate and transfer proof are in private scratch
`main-merge-claude-u8H9Ci`.
The initial generated `CLAUDE.md` SHA-256 is
`86e85df40188ed189c723dc8149fc040d0d1f0f047613cfe2603db7bb34d6140`.
After the user adds the standing issue-reporting requirement,
`main-merge-claude-LUEW9u/standing-rule-verification.json` verifies the fresh generated file:
`eeaae1583f726f91c911e677215db962356acd31f69e4ab9d279ec1550956f9d`.
The standalone generator is frozen under
`node_modules/.frozen-merge-claude-generator-kWWHmw`.
This is a scoped generator invocation,
not a full root file-enforcer run.

Lock generation uses `mise run prepare:pnpm:install` with native lock-only,
offline,
no-lifecycle-script and read-only-store flags.
The container has 2 GiB RAM,
2 GiB swap,
two CPUs,
512 PIDs and no network.
Repository source and existing dependency directories are mounted read-only;
only the lockfile's parent replacement path and private diagnostic paths are writable.
The initial attempt lacks the separate package metadata mirror.
The next attempt mounts that existing mirror read-only and succeeds.
`main-merge-lock-generation-dZaFeK` records peak `250773504` bytes and zero OOM/PID events.

The generated lock adds 90 lines relative to main:
translation-repair's importer,
its catalog entries,
`pinyin-pro` records and the retained `nano-spawn` dependency.
It changes no existing main lock entry.
Main's catalog now selects `satteri@0.10.5`,
TypeScript `7.0.2` and other updated dependency versions.
Historical preparation qualification remains bound to its older immutable artifacts;
this merge does not qualify new dependency builds.

## Standing issue reporting and unreviewed changes

The user requires GitHub issues for adjacent repository defects,
including local workarounds or fixes,
and requires those records to state that the changes have not been reviewed.
Rule XIR and regenerated `CLAUDE.md` are committed in `7c5810004`;
Markdown verification passes.

The following issues remain open with `needs-triage`.
Every issue's title,
body and explicit unreviewed status are read back after creation.
The creation queue is serial,
pauses between mutations and performs no automatic retry.

- [#509](https://github.com/Aquaticat/Monochromatic/issues/509):
  logger callback-observation workaround and native fault fixtures.
- [#510](https://github.com/Aquaticat/Monochromatic/issues/510):
  logger concurrency assertion changed from elapsed time to an explicit completion barrier.
- [#511](https://github.com/Aquaticat/Monochromatic/issues/511):
  TOML unit tests moved to built artifacts and existing unstable test seams.
- [#512](https://github.com/Aquaticat/Monochromatic/issues/512):
  TOML readonly observation views and parser ownership boundaries.
- [#513](https://github.com/Aquaticat/Monochromatic/issues/513):
  Markdown merge export and comment reconciliation.
- [#514](https://github.com/Aquaticat/Monochromatic/issues/514):
  inherited TSDoc prefix and catch-binding conventions.
- [#515](https://github.com/Aquaticat/Monochromatic/issues/515):
  missing dependency reported through the trust bundle's self-contained check.
- [#516](https://github.com/Aquaticat/Monochromatic/issues/516):
  Markdown policy dependency failure truncated into a report-parse diagnostic.

The local fixes and workarounds have not been reviewed.
Passing tests or formatter output is not review.

The repository-wide `initPromise` search also updates existing issue
[#254](https://github.com/Aquaticat/Monochromatic/issues/254),
rather than creating a duplicate.
The active Done application's `src/lib/db-migrations.ts`
and paused consumers still import the removed default-logger readiness export.
Those consumers are not changed or qualified by this merge check.

## Final selected-package verification

`main-merge-package-checks-63k9ap` passes fresh builds for fs-path,
async-time,
logger,
module-test,
Markdown-lint and TOML-edit.
Those packages plus TOML-edit's fuzz sidecar pass type checks,
complete unit tasks and read-only Oxlint tasks.
Each lint run reports zero warnings and zero errors with 485 rules.
The sidecar runs its bounded unit mode,
not a time-budgeted fuzz campaign.

The container retains the 2 GiB RAM,
2 GiB swap,
two-CPU,
512-PID and no-network limits.
Peak memory is `746094592` bytes with zero OOM/PID events.
The measured package source/configuration inventories remain unchanged.
The actual runtime ledger records Node `26.8.2`.

`main-merge-package-checks-LDVNaW` repeats the complete selected matrix
with committed `mise.lock` bytes mounted read-only over the owner-modified file.
This separates committed-tree verification from preservation of local tool versions
without changing the host file.
The committed lock SHA-256 is
`f0e844684cc0ba3176b47c85e47d50ca505b134be352b3e7e47b472e88b350ba`.
All phases pass again;
root configuration hashes and the owner lock hash remain unchanged.
This run peaks at `680280064` bytes with zero OOM/PID events.

Dependency directories are read-only overlays from main's installed tree.
This is source compatibility evidence against that overlay,
not proof of a fresh installation from the merged lockfile
and not a new translation-repair full-suite or native preparation qualification.
The worktree bootstrap also restores 189 missing declared workspace links,
replacing no existing links;
that operation is not a native package-manager install.

`main-merge-lock-generation-JLN4yl` passes native PNPM `12.3.4`
frozen-lock and supply-chain verification for all 773 package entries.
The lockfile bytes are unchanged.
This remains a metadata-only check,
not package installation.

Cold checks first refuse absent abbreviated metadata,
then absent full metadata.
A separate private package acquires abbreviated registry metadata with scripts disabled,
without mounting the repository or real home.
The no-network verifier mounts that cache read-only
and exposes the existing filtered-full mirror at its full-metadata lookup path.
The release source explicitly accepts filtered-full data in its shared in-memory trust lookup.
That supports the data projection,
not an assertion that PNPM documents the on-disk directory alias.
The wrapper mapping is instead demonstrated by the actual successful offline command;
it preserves the original bytes rather than manufacturing cache records.
No trust or age policy is disabled,
and the final check does not inherit a cached lockfile-verification verdict.

`main-merge-metadata-proof-oVKGP8` subsequently retains 1492 metadata files
with `215155142` bytes.
The `main-merge-lock-generation-IWD4cP` rerun uses that owned snapshot,
passes all 773 entries,
and verifies metadata and executable hashes before and after the invocation.
This is cached-metadata compatibility,
not current-registry verification or installation.
See [the source-traced PNPM record](../troubleshooting/pnpm-offline-verifier-metadata-mirrors.md).

`main-merge-package-checks-A7pdf5` also exercises the actual built Markdown CLI:
help,
semantic wrapping,
tag-like paragraph preservation,
older quote wrapping and usage refusal all pass.
The stdin consumers use a disposable directory,
and the CLI bytes are unchanged after invocation.

A native Mise dry-run against the reconciled private lockfile also succeeds.
Its bytes and the unstaged owner file both retain SHA-256
`6a3dcb5cbacd22f38feb141320f707ca41aa504b01df9197567ea4bf65ba4b12`.

## Declaration audit boundary

`main-merge-export-audit-kDOYn8` parses the 28 TypeScript files
among the 38 paths changed by both parents.
It inventories top-level exports and literal test descriptor names,
with positive controls.
It is not function-body equivalence or a completed code review.

The declared-name differences are explained:
main's `b333197d5` deliberately removes the default singleton `initPromise` export,
and two test descriptors are renamed.
`main-merge-renamed-tests-g0Q6lv` verifies the renamed callback bodies
have unchanged ASTs after removing locations and comments.
The scanner's source and generated mirror retain main's indexed ASCII validation
plus the repair branch's diagnostic text.
The initial Git CLI merge retains main's authoring entry
and the repair `resolveGit` export/subpath.
The subsequent target lookup proves the latter is invalid after main's resolver move;
`Verified merge scope` records its reconciliation.
The remaining jointly changed documents,
configuration and moved TOML fuzz helper are classified separately,
not inferred from the AST-name result.

## Evidence retention and cleanup

`merge-retention-1vDapK` retains the merge,
resolver and style evidence in private agent scratch.
The manifest records 10291 files,
`537852297` bytes,
3148 directories,
916 empty directories and 977 symlinks.
There are no special entries or foreign-owned entries.
Mutable build outputs from the selected packages are copied;
the copied regular files are hash-checked against their sources.
Bind-mounted diagnostic files remain retained in place.
CID bytes are copied before Podman can remove their original paths.

At `2026-09-14T20:07:39.019Z`,
cleanup removes 51 exact owned nonrunning container IDs,
including the retention workers.
Every ID is inspected before removal and checked absent afterward.
No force or prune operation is used.
All unselected container IDs remain present;
40 containers remain outside this cleanup scope,
including older comparison diagnostics and unrelated workloads.
The worktree,
corpus,
owner lock backup and frozen preparation runtimes are not removed.

The first retention attempt stops at a creation-record filename mismatch;
its retained output is preserved.
The successful retainer supports both recorded `launch.json` and `invocation.json` forms
while requiring the exact CID path and container name from creation arguments.
Final root sentinel checks find no ignored `HEAD`,
`config`,
`hooks`,
`objects` or `refs` artifact to clean.

## Paused repair work

Tasks55,
54 and51 remain incomplete while task56 performs this merge.

Task55 adds `observeLoggerCallbacks` to the existing logger package boundary.
Commits `fb0743f44` and `ea3087d9c` contain its implementation,
exports and built-artifact tests.
Subsequent selected-package verification passes their build,
type check,
complete unit task and read-only lint.
README completion,
consumer integration and their remaining task-specific acceptance still belong to tasks55 and54.
The implementation remains unreviewed in #509.
No test-only public comparison export is introduced.

The selected task54 remedy treats callback exceptions as observable telemetry,
not cancellation or evidence authority.
The adapter must preserve lazy lookup,
receiver,
message order and every requested attempt,
while recording only a bounded canonical frozen callback-name set.
It must not inspect or retain thrown values.
Explicit flush remains explicit;
no extra flush or retry is added.
This is not protection against blocking,
process exit,
filesystem mutation or a void method returning an unobserved rejecting Promise.

Comparison integration is not implemented.
It must expose the final snapshot after success or terminal warning,
preserve the primary names-only failure and owned directory,
and permit only actual failure-record I/O refusal to supersede that failure as `storage`.
The creation-info regression must become matched success with observable telemetry degradation,
not a fabricated storage failure.
Additional final-log,
post-sync-log and getter controls remain required.

The earlier comparison checkpoint passes zero full-scope lint and 632 observed default-suite entry processes,
but commit `13870bd70` subsequently exposes ten logging-retention regression failures.
Those are recorded in `devtest-iU1fZl/logging-red-reading.json` under private scratch.
They are pre-existing at this merge,
not introduced by it.
The red verifier checks exact test names and ordinary assertion classification;
strengthen its exact matcher/location checks before guard-removal proof.

## Merge checkpoint and corrective scan

Merge commit `9c6cc09079397eb45e4f645c718163347a72d8bc` has the expected repair and main parents.
The Git index has no unmerged source entries after that commit.
That was insufficient verification:
a full tracked-text marker scan subsequently finds two unresolved comment hunks
in `package/cli/markdown-lint/src/rule/semantic-line-breaks.ts`.
They were missed after a truncated file read.
The correction keeps the space-consuming fix documentation and main's comment layout.
It is a follow-up commit,
not an amendment of the merge commit.
`main-merge-residual-markers-before-20260914.txt` under private scratch records the exact locations.
`main-merge-marker-verification-20260914.json` subsequently checks all tracked working-tree text,
not only the formerly unmerged paths.
Its positive control finds the committed markers;
the corrected working-tree scan has no matches.

Proposed `AGENTS.md` addition under completion verification:
"Before completing a merge,
scan tracked text for conflict markers.
An empty unmerged index proves only staging state,
not hunk resolution."
This proposal is not applied to `AGENTS.md`.

The local lockfile reapplication conflicts on Wrangler,
pnpm,
Rust and uv versions.
The owner versions win those conflicts.
The complete three-way result also retains main's GNU Hyperfine asset metadata.
It differs from the original backup only in those Hyperfine metadata fields,
so restoring the original bytes alone would lose a main change.
A private Mise generation refreshes Hyperfine metadata from the original lockfile.
Its whole output matches the intended three-way result exactly before transfer.
The restored file SHA-256 is
`6a3dcb5cbacd22f38feb141320f707ca41aa504b01df9197567ea4bf65ba4b12`.
The Git index is reset to the merge commit's lockfile without staging the local changes.
`main-merge-lockfile-VyH4kO/restoration-complete.json` records that proof.
The exact original backup remains retained.
The owned temporary stash is removed after restoration verification;
`stash-cleanup.json` verifies that both unrelated stash identities remain unchanged.

## Remaining merge work

Resume task55's logger documentation and utility acceptance,
then task54's comparison integration.
Keep the reconciled owner lockfile unstaged.
The comparison's known logging regressions remain outside the completed merge scope.
Check both merge parents,
no unmerged entries or accidental duplicate exports,
generated-file equality and local-change preservation.
Record actual test outcomes without presenting the known logging regressions as passing.
Retain diagnostic evidence before removing exact owned stopped containers.
No corpus edit,
paid preparation,
writer calibration or new full-entry pass is authorized or performed.
