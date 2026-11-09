# Shadow repository and replay route findings on main

## Scope and evidence, 2026-11-09T06:08:41-05:00

Audited `main` at `566be9a823d58f9532b14a48d54980a8e8b4fec4`,
whose commit timestamp is `2026-11-09T05:45:38-05:00`.
The audit worktree started at `b38de34bb23ba5fd29fcbc70e8be218ff9746edd`.
`git diff --exit-code 566be9a823d58f9532b14a48d54980a8e8b4fec4 HEAD --`
with `package/git-policy/cli/src/native`,
`package/git-policy/cli/SPEC.md`,
and `doc/decision/cli-git-concurrent-commits.md` returned zero:
the audited production sources and those records match that `main`.
No unmerged implementation branch was examined.
No production code was changed.

The owner requirements supplied with this audit are:

- "Commits should never fail."
- "Auto merging should never fail."
- "never discard user data"
- "nothing should ever wait forever or indefinitely"

The first requirements are also recorded in `package/git-policy/cli/SPEC.md:2160`.
`doc/handover/cli-git-rust-implementation.md:5561`,
"Open review findings",
was checked before reporting new findings.
The recorded autostash loss and the general deadline/branch-write work are not new findings here.

All reproduced results use real Git `2.56.0` and the unmodified production functions through temporary Rust tests.
Histories were generated with `hash-object`,
`mktree`,
`commit-tree`,
and `update-ref` in disposable repositories under this worktree's `.audit-shadow-replay/`.
The generators give every child a 15-second deadline.
Probe commands have an outer deadline and a cgroup with CPU quota `400%`,
`MemoryMax=8G`,
`TasksMax=256`,
and `IOWeight=10`;
generators use lower limits.
No synthetic load or mutation campaign was used.
The probe tests are not part of the committed change.

## What the route does and who calls it

`native/shadow/` holds two related mechanisms,
not one continuously used shadow repository.
All source paths in this document are relative to the repository root.

- A commit that needs native Git,
  including configured signing or an editor,
  uses `package/git-policy/cli/src/native/landing/service_native.rs:200`.
  `build_with_git` creates a scratch shadow with `create_shadow_repository_carrying`,
  runs native `git commit` against a private index,
  and reads the prepared object.
  The current scratch path is `<git-dir>/monochromatic-git-policy-cli-service-<id>/shadow`,
  not the historical per-transaction registry path still described in some source comments.
- A commit sent back after its branch moved enters
  `package/git-policy/cli/src/native/landing/service_replay.rs:245`,
  `merge_captured`.
  This first calls Git's `merge-tree` with the original preparation base.
  Only conflicted paths enter capture ordering and subsumption.
  A synthetic base settles paths judged contained or ordered;
  another `merge-tree` runs,
  followed by the word-level pass if necessary.
  This current caller passes the real common Git directory and real object store to the helpers named "shadow".
  Replay is not confined to the native preparation shadow.

The accepted decision in `doc/decision/cli-git-concurrent-commits.md:79` says:
"when the landed change for that path
(its diff from the preparation base to the new `HEAD`)
applies in reverse to the prepared version,
the prepared bytes already contain it and land as they are".
Its implementation-time amendment at `:321` permits strict reverse application
or a one-sided extension of the landed hunks.
The capture-order decision at `:330` chooses later captures from the same worktree;
unknown captures still go through subsumption and Git.
The reproductions use unknown capture order,
so an intentional later-capture override cannot explain their results.

## Findings ranked by risk

### P1: A non-ASCII conflict can silently revert another file's landed change

Status:
**reproduced** through `service_replay::merge_captured`,
including the final word-level pass.
The wrong result is an accepted tree,
not merely a malformed diagnostic or an intermediate synthetic base.
This violates "never discard user data":
the new tree drops the effect of an existing landed edit without naming that file.
The landed commit and its objects remain in history;
this is not a claim that its historical blob was deleted.

Relevant code:

- `package/git-policy/cli/src/native/shadow/replay_shared_paths.rs:207`
  decodes each pathname byte as a Latin-1 character.
- `package/git-policy/cli/src/native/shadow/replay_subsumption.rs:160`
  formats that carrier string and calls `into_bytes()`,
  encoding it as UTF-8 instead of reversing the Latin-1 carrier.
  The deletion and exact-entry records at `:154` and `:173` have the same boundary error.
- `package/git-policy/cli/src/native/shadow/replay_subsumption.rs:219`
  reads the original base into a private index;
  `:230` installs those wrongly named records with `update-index -z --index-info`.
- `package/git-policy/cli/src/native/landing/service_replay.rs:251`
  lists paths before filtering them to the originally conflicted set,
  and `:285` uses the synthetic base for another whole-tree merge.

Exact sequence:

1.  Create a base commit with `é.txt` containing `a\nb\nc\nd\n`
   and the distinct filename `Ã©.txt` containing `unrelated base\n`.
   Their UTF-8 filename prefixes are respectively `c3 a9` and `c3 83 c2 a9`.
2.  From that base,
   make the landed commit change `é.txt` to `a\nB\nc\nd\n`
   and change `Ã©.txt` to the same bytes `a\nB\nc\nd\n`.
3.  Independently from the base,
   prepare `é.txt` as `a\nB\nC\nd\n`.
   Leave `Ã©.txt` untouched at `unrelated base\n`.
4.  Real `git merge-tree --write-tree -z --merge-base=<base> <landed> <prepared>`
   exits `1` for the contained adjacent edit on `é.txt`.
   Its output tree preserves `Ã©.txt` as `a\nB\nc\nd\n`.
5.  Call the production `merge_captured` with those histories and capture stamp zero.
   It accepts a `Combined` result:
   the word-level pass resolves the original `é.txt` conflict,
   but `Ã©.txt` has reverted to `unrelated base\n`.

The equality between the landed blobs is essential,
not an accidental test shortcut.
The synthetic-base record meant for `é.txt` instead makes the base entry for `Ã©.txt`
equal to that file's landed entry.
Git therefore sees no landed-side change at the unrelated filename
and takes the prepared side's original bytes there.
The original `é.txt` conflict remains for the word-level pass,
which resolves it successfully and returns the already-corrupted whole tree.

Quoted output:

```text
path-collision: native merge exit=Some(1)
path-collision: production result=Combined
path-collision: native bytes="a\nB\nc\nd\n", production bytes="unrelated base\n"
assertion `left == right` failed
  left: [117, 110, 114, 101, 108, 97, 116, 101, 100, 32, 98, 97, 115, 101, 10]
 right: [97, 10, 66, 10, 99, 10, 100, 10]
```

The full accepted tree contains both original filenames;
this is not a missing-path or failed-`show` artifact.
An otherwise identical history using `a.txt` and `b.txt` passes:
Git's output tree and production replay both retain `b.txt` as `a\nB\nc\nd\n`.

Suggestion:
keep repository paths as bytes through the synthetic-index writer,
or explicitly invert the Latin-1 carrier at that boundary.
Add a consumer-level control with distinct real filenames that alias under the current encoding,
checking both the combined file and the unrelated file's content and mode.
Do not treat adding another UTF-8-only parser test as sufficient.

### P1: A real-gitdir signing include disappears in the shadow and an unsigned commit succeeds

Status:
**reproduced** through the production invocation capture,
`create_shadow_repository_at`,
`run_native_commit_within`,
and `read_prepared_commit`.
No signer failure or timing race is involved.

The accepted preparation decision in `doc/decision/cli-git-concurrent-commits.md:35` says:
"Git then owns hooks,
the editor,
templates,
message cleanup,
and signing."
Current `package/git-policy/cli/SPEC.md:7473` supersedes the hook portion by supporting no external hooks;
it does not retire configured commit signing.
The shadow route must not change a person's signing request into a successful unsigned commit.

Relevant code:

- `package/git-policy/cli/src/native/shadow/shadow_config.rs:244`
  includes the real common config,
  but does not preserve the invocation's effective values from conditional gitdir includes.
- `package/git-policy/cli/src/native/landing/service_prepare_changes_config_and_pathspec_steps.rs:113`
  routes effective `commit.gpgSign` configuration to native Git.
- `package/git-policy/cli/src/native/commit/commit_preparation.rs:141`
  substitutes `--git-dir=<shadow>` in that native command.
  `:194` runs it without restoring signing values that applied only to the original Git directory.
- `package/git-policy/cli/src/native/landing/service_native.rs:214`
  uses this creator and native preparation in the shipped path.

Exact sequence:

1.  Create an ordinary repository and a disposable SSH signing key with no passphrase.
2.  Put `commit.gpgSign=true`,
   `gpg.format=ssh`,
   and `user.signingKey=<disposable key>` in a separate configuration file.
3.  Include that file from the repository config only under
   `[includeIf "gitdir:<absolute repository path>/.git"]`.
4.  Native `git commit --allow-empty --message=<message>` succeeds.
   Require that its raw commit header contains `gpgsig`:
   this positive control was asserted,
   not merely printed.
5.  Capture the same repository with the production capture functions,
   create its shadow,
   and initialize a private index from `HEAD`.
6.  Run production native preparation with `--allow-empty --message=<message>`,
   no `--no-gpg-sign`,
   and no command-line signing override.
   It exits `0`,
   and `read_prepared_commit` reports `signed=false`.

Quoted output:

```text
real Git honors exact gitdir include: signed=true
exact gitdir include: native signed=true, shadow exit=0, shadow signed=false
the shadow must not silently turn off the person's configured signing
```

The source-level explanation is that the exact gitdir condition no longer matches the substituted shadow directory.
The fixture has no other signing configuration,
and `format_shadow_config` emits no signing override.
The signed/unsigned disagreement is measured;
this account of conditional matching is the interpretation of those inputs and the changed Git-directory selection.
The existing conditional-include test in
`package/git-policy/cli/src/native/shadow/shadow_create_tests.rs:82`
checks `onbranch:main`,
which is preserved by the symbolic shadow `HEAD`;
it does not exercise an exact gitdir condition.

Suggestion:
preserve the relevant effective native-commit configuration from the invocation context,
with its intended precedence and path-resolution semantics,
rather than assuming that including the real file recreates every condition.
A regression control should require a signed native positive control and a signed production shadow commit.

### P2: An unrelated Git-valid non-UTF-8 tag blocks native preparation

Status:
**reproduced** through production invocation capture and `create_shadow_repository_at`.
This is a visible failure,
not silent data loss,
so it ranks after the accepted wrong results.
It touches the owner's "Commits should never fail."
The failure is specific to commits using native preparation,
not every commit the embedded service can build without a shadow.

`package/git-policy/cli/src/native/shadow/shadow_refs.rs:89`
decodes the complete `for-each-ref` listing as UTF-8
and returns `GitFailure("git for-each-ref returned non-UTF-8 output.")` on any invalid sequence.
`write_shadow_refs` calls it at `:205`,
and the creator propagates its failure at
`package/git-policy/cli/src/native/shadow/shadow_create.rs:370`.
The production caller is the native builder at
`package/git-policy/cli/src/native/landing/service_native.rs:214`.

Exact sequence:

1.  Create a repository on `main` with an empty base commit.
2.  Use `git update-ref --stdin` with raw input bytes to create the valid ref name
   `refs/tags/nonutf-` followed by byte `ff`,
   pointing at that base.
   The non-UTF-8 name is unrelated to `HEAD` and to the requested change.
3.  Native `git commit --allow-empty --message=<message>` succeeds in the same repository.
4.  Capture the repository and base through production capture,
   both successfully,
   then request a production shadow.
5.  Shadow creation fails at the complete-ref snapshot's UTF-8 decode.
   A commit needing this route,
   such as one with an editor,
   cannot reach native preparation even though Git accepts the repository and the commit.

Quoted output:

```text
native commit beside non-UTF-8 tag: exit=0
non-UTF-8 tag: native subject="native control beside non-UTF-8 tag\n"
shadow creation=Err(GitFailure("git for-each-ref returned non-UTF-8 output."))
valid non-UTF-8 reference should not block a commit
```

Suggestion:
parse and reproduce ref names as bytes on Unix,
keeping diagnostic text separate from the bytes written to the ref snapshot.
A control should put a non-UTF-8 ref outside the target branch and require native shadow creation to succeed.

## Creation, shared state, interruption, and waits

The code keeps `HEAD`,
refs,
reflogs,
index,
config,
and the shadow object directory private.
`package/git-policy/cli/src/native/shadow/shadow_links.rs:67`
excludes conclusion state and names ending in `.lock` too.
Other common-directory entries,
including `info`,
`rr-cache`,
`lfs`,
and `modules`,
are links into the real repository.
`config.worktree` and a linked worktree's sparse-checkout file are copied where applicable
(`shadow_links.rs:217`).
This is shared filesystem state,
not an isolation boundary against arbitrary writes.

Native preparation deliberately sets `GIT_OBJECT_DIRECTORY` to the real store
(`package/git-policy/cli/src/native/landing/service_native.rs:224`;
`package/git-policy/cli/src/native/commit/commit_preparation.rs:202`).
It therefore writes real objects even though its refs and index are private.
Replay likewise writes content-addressed objects in the real store through its current caller.
No operation observed in these probes deletes a real object or a linked payload.

For the current native route,
`Scratch::drop` removes the scratch tree
(`package/git-policy/cli/src/native/landing/service_prepare_ready_shapes.rs:36`).
After a successful preparation,
the client also hands the scratch directory to a service watch
(`package/git-policy/cli/src/native/landing/service_commit.rs:222`).
Connection closure queues its release
(`package/git-policy/cli/src/native/service/service_server_connection_handling.rs:305`),
and the service removes released trees (`service_server.rs:470`).
A kill during creation can leave a partial shadow,
loose objects,
and private indexes.
The next invocation gets another random scratch identity;
it does not use directory existence as proof of a completed commit.
I did not establish eventual collection for a kill before the scratch watch is registered,
or for a simultaneous client/service death.
Those are limits of this audit,
not reproduced recovery guarantees or new data-loss findings.

Replay's Git reads and object writers go through the watched transaction runner
(`package/git-policy/cli/src/native/transaction/transaction_git.rs:374`;
`transaction_git_step_bounds.rs:53`).
The native preparation's no-editor path has its own ceiling.
An editor-open native commit intentionally has no ceiling:
the newer owner answer is "limit unless editor"
and "no ceiling,
only the waiting notice" with an editor open
(`doc/handover/cli-git-rust-implementation.md:7525`).
That exception must not be misreported as a newly discovered violation of the general no-indefinite-wait requirement.
The helper files contain no additional explicit wait loop independent of those runners.
Direct filesystem creation,
copy,
and deletion calls are synchronous;
this audit did not prove bounds for an unresponsive filesystem syscall.

## Verification runs

The filtered nextest command was
`mise run //package/git-policy/cli:native:test:host -- audit_probe --success-output=final --no-fail-fast`.
Its named output ran `9` probes:
`7` passed;
`audit_path_collision_preserves_the_unrelated_landed_edit`
and `audit_exact_gitdir_include_keeps_required_signing_enabled` failed with the quoted wrong results.
A later filtered run with the separate name filters `audit_non_utf8 audit_partial_shadow`
ran `2` probes:
`audit_partial_shadow_cleanup_does_not_follow_shared_links` passed,
and `audit_non_utf8_tag_does_not_prevent_native_shadow_creation` failed with the quoted diagnostic.
Thus each reported defect has a red assertion against an unmodified production seam.
No full suite was run.

One attempted filter contained a literal `|`,
selected no tests,
and ended with nextest's `error: no tests to run`.
That run is not counted as verification;
the corrected separate filters produced the named results.
An initial launcher was stopped before execution because it lacked the explicit fixture-root environment.
No result from that launcher is counted either.

The required document tasks both returned zero:
`mise run //:format:monochromatic-lint -- doc/planning/cli-git-shadow-and-replay-route-findings-on-main.md`,
then `mise run //:lint:monochromatic-lint -- doc/planning/cli-git-shadow-and-replay-route-findings-on-main.md`.
Pandoc's CommonMark-to-HTML output was read to check the rendered headings,
sequences,
code spans,
and emphasis.
The uncommitted probe sources,
generators,
and full outputs are preserved under `.audit-shadow-replay/`,
not under the crate's source tree.
`git diff --exit-code -- package/git-policy/cli` confirmed that temporary crate edits were removed.

A cross-vendor Kimi review challenged the reachability,
positive controls,
and interpretation.
The conflict status was asserted,
the full resulting tree was inspected,
and configured native signing was proved before accepting the failures.
The interhunk hypothesis was not supported by the measured Git output.
A separate DeepSeek review returned no visible text and supplies no review evidence.

## Verification limits

The probes call shipped functions at the captured-history and native-preparation seams,
not the installed wrapper or its live service.
They establish wrong replay trees and wrong native prepared objects,
not a timed concurrent invocation or a final service landing.
No production fix was attempted.
Windows and macOS were not exercised,
as requested.

The submodule path was read:
`replay_shared_paths.rs:328` accepts only regular-file modes for text subsumption,
so gitlinks and symlinks fall back to Git rather than the text heuristic.
Symlink and binary conflicts were exercised;
a real submodule checkout,
SHA-256/reftable repositories,
custom merge drivers,
all attribute combinations,
and every possible interrupted-write position were not exercised here.
No general safety claim follows for those surfaces.

## Checked and found sound

- At the interrupted-creation seam after skeleton creation and production shared-entry linking,
  `remove_shadow_repository` removes the partial tree without following its `info` link.
  The test first reads the real sentinel through the shadow link,
  then checks the original sentinel bytes after deletion.
  Output:
  `partial shadow: shared sentinel intact=true, shadow removed=true`.
  This exercises the removal helper,
  not a whole client/service crash-recovery cycle.
- The ASCII equivalent of the filename-collision history retains the unrelated landed edit.
- The adjacent contained text edit is retained with exact CRLF bytes.
- A landed `100755` mode survives a contained text replay;
  both mode and bytes were asserted.
- Different NUL-containing binary blobs stay a conflict rather than becoming text subsumption.
- Different symlink targets stay a conflict.
- `diff.relative=true` with a subdirectory invocation does not turn the tested same-line conflict into success.
- The suspected `diff.interHunkContext=1` counterexample was rejected by measurement:
  the actual zero-context `diff-tree` output still had separate hunks,
  and production returned `a\nB\nD\nD\ne\n`,
  retaining both edits through the word-level pass.
  It is not a finding.
- The indexed text diff explicitly disables external diff and text conversion,
  forces text only after its own binary exclusion,
  disables rename detection,
  and uses decimal synthetic filenames instead of user filenames in patch headers
  (`package/git-policy/cli/src/native/shadow/replay_subsumption_text.rs:171`).
  The raw shared-path listing uses NUL delimiters and disables rename detection
  (`replay_shared_paths.rs:181`).
  The path-encoding defect is at the later index-record writer,
  not those delimiters.

## Fixes landed, 2026-11-09T07:47:32-05:00

Each finding was reproduced first with real Git `2.56.0` beside the wrapper and against `main` at
`566be9a823`, then fixed on the branch
`fix/git-policy-cli-shadow-replay-keeps-distinct-files-and-signing`.
Every control was also checked by removal: with the fix taken out again,
the control fails with the wrong result this document quotes for that finding.

### The non-ASCII conflict that reverted another name

Commit `2bb6958077` follows its control, `ae249405ed`.
A repository path now stays the bytes Git listed from the raw shared-path listing
through the `update-index --index-info` record,
so no boundary re-encodes a name.
The Latin-1 converters the replay used are gone.
Controls:
`landing::service_replay::tests::a_conflict_on_one_name_keeps_the_other_names_landed_edit`
reads both names' bytes and modes from the accepted tree,
and
`shadow::replay_shared_paths::tests::raw_records_keep_path_bytes_that_are_not_text`
keeps a name that is not UTF-8 through the parser.

### The signing request that an exact `gitdir` include lost

Commits `c268e5d579`, `ebfc4c6d99` and `b7a51bd742`.
The capture's configuration snapshot already holds what Git resolved for the invocation,
so each key native `git commit` reads now travels to the shadow as a `-c` setting,
the last entry of a key being the value Git would have used.
One list, in `src/native/commit/native_commit_configuration.rs`, now says which keys those are,
and the route to native preparation reads the same list.
Control:
`commit::commit_preparation::tests::signing_from_an_exact_gitdir_include_reaches_the_shadow`,
which requires real Git to sign through the include before it requires the production preparation to sign.

### The non-UTF-8 tag that blocked native preparation

Commit `cba158fe73` follows its control, `b45b936337`.
A ref name and a symbolic target stay bytes through the listing, the `packed-refs` snapshot,
the reftable `update-ref --stdin` transaction and the `symbolic-ref` call.
An object name must still be hexadecimal text, and any other listing shape is still refused whole.
Controls:
`shadow::shadow_create::tests::a_ref_name_that_is_not_text_does_not_stop_the_shadow`,
`shadow::shadow_refs::tests::the_files_snapshot_keeps_a_name_that_is_not_text`
and
`shadow::shadow_refs::tests::the_reftable_snapshot_keeps_a_name_that_is_not_text`.

The "Not determined" limits of the audit stand unchanged:
submodules, SHA-256 and reftable commits, custom drivers, attribute combinations,
interruption before the scratch watch is registered, simultaneous client and service death,
crash recovery and unresponsive filesystem bounds were not exercised here.
