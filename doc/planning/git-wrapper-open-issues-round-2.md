# Git wrapper open issues, second pass

What:
 the open issues that name the Git wrapper,
 read again against this branch's base,
 with what today's code does,
 what changed,
 and what is left for someone else.

Why:
 the first pass (`doc/planning/git-wrapper-open-issues-against-main.md`)
 classified 427 issues,
 verified some of them only by a surviving quote,
 and left six untested-arm reports plus one lossy-path audit for a fresh reading.

## Method

Each issue was read with `gh issue view <number> --json title,body`,
 and its claim was checked against the code at `761a2ece55`,
 this branch's base,
 by reading the named module
 and the controls that are supposed to cover it.
`origin/main` has moved on to `c6a356670b` since then,
 and this branch merges nothing into it.
A claim that no longer matches a module was checked with `git log -S` for the commit that
 removed it,
 so "gone" names the commit rather than only the absence of the text.
Every new control was proved by removing its guard and quoting the failure
(`AGENTS.md` `GFP`).

## Fixes

### `#4576` duplicate names, one lookup per manifest

Held:
 the first pass's line number was right;
 `duplicate_name` compared each manifest with every earlier one.
Fixed in `97d0e4ef77`:
`src/native/dependent_version/dependent_version_states.rs` builds a table from name to the
 first state that declared it,
 so one manifest costs one lookup and the whole workspace is read once (`RG2`).
The entry is never replaced,
 so the reported pair is still the first later state whose name an earlier one declared.

### `#4563` one dependency field, one lookup per member

Held:
 `dependency_names` tested each name against every name the field had already contributed,
 at `src/native/dependent_version/dependent_version_manifest.rs`.
Fixed in `5b028c6c00` with a set created per field,
 which keeps the existing rule that two different fields may both declare one name
 (`["b", "c", "b"]` in
 `dependent_version::dependent_version_manifest::tests::keeps_dependency_names_unique_within_a_field`).

### `#6003` one object's keys, one lookup per member

Held:
 `member_keys` used `keys.contains(&key)`,
 at `src/native/config/config_values.rs`.
Fixed in `5236c0cd95` with a set companion.
Keys are still decoded and judged one at a time,
 so a repeated key before an undecodable one still reports the repeat.

### `#2135` four `expect(true).toBe(true)` suites

Held:
 all four sites were still there,
 and the Windows-only case reported a pass for a body that never ran.
Fixed in `aa04adecdc` and `6ef1dcbcf2`:

- `e2e/process-fixture.unit.test.ts` records uncaught exceptions during the grace window and
  asserts there were none,
  which is the process-level failure the window exists to catch.
  The capture is a factory returning a `Disposable`,
  because Oxlint bans `try`/`finally` in this repository.
- `prefer-readonly-parameter-type/src/typescript-sync-adapter.unit.test.ts` uses the harness
  `skip` flag with its reason,
  so this host reports `SKIP` instead of a pass.
- `music-player/desktop-e2e/src/press.unit.test.ts` asserts the press reached the session and
  nothing else did.
- `translation-repair/shared-contract/src/bench-wire-model.unit.test.ts` names the accepted
  roster once and asserts it is nonempty,
  so the accepted case is a case the rule had to allow.

### `#2108` two `pub fn quote`

Held:
 both functions are still there,
 at `src/native/foundation/json_record.rs` (serde) and
 `src/native/dependent_version/dependent_version_fixture_json.rs` (UTF-16 units).
Fixed in `1fc2cd75d4` with the arrangement the first pass recommended:
 the fixture side owns the byte-for-byte `JSON.stringify` spelling of fixture output,
 because a fixture string can hold a lone surrogate and Rust `&str` cannot,
 and the release path keeps `serde_json`.
No code changed.

### `#4530` an inherited `SIG_IGN`, untested arm

Held:
 `install_one` returns `Ok(None)` for a signal the parent ignores,
 at `src/native/process/signal_relay.rs`,
 and no control reached it.
Fixed in `643462a0ae`:
`process::signal_relay::tests::an_inherited_ignore_is_not_replaced_by_the_guard` sets
`SIGHUP` ignored and asserts the disposition is still `SIG_IGN` while the guard lives and
 after the drop.

### `#4585` `locate_hooks`, bare-repository fallback

Held:
 the fallback arm `Ok(_) | Err(_) => PathBuf::from(git_directory)` at
 `src/native/health/doctor.rs` had no bare-repository control.
Fixed in `1aa121149b`:
`health::doctor::tests::a_relative_hooks_path_in_a_bare_repository_resolves_against_the_git_directory`
makes a bare repository,
sets a relative `core.hooksPath`,
 and asserts both the common directory and the resolved path.

### `#4514` untested empty shared-path listing

Not as filed:
`decide_shared_paths` and its empty-candidates early return are gone,
 with `replay_capture_history.rs` in `72e0534cca`.
Its successor `list_shared_paths` (`src/native/shadow/replay_shared_paths.rs`) always lists
 both sides and can still return nothing.
The empty answer is now pinned in `77b8fd4039` by
`shadow::replay_shared_paths::tests::sides_that_changed_different_paths_share_none`.

## Untested-arm reports whose arm is gone or covered

### `#4505` `refuse_unreconciled_landing`

Not as filed:
`recovery_contradiction.rs` and its controls were deleted in `72e0534cca`.
The record set it contradicted (`preparing.json`,
 `landing-1.json`,
 `ref-updated.json`) is gone with that commit;
`src/native/transaction/transaction_journal.rs` says so in its module doc.
The decision it guarded survives as `landed_below` in
`src/native/landing/service_land_finish.rs` (exit status 1 is "not below",
 any other failure is no answer),
 and the no-answer arm is covered by
`landing::service_land::finish_git_failure_tests::a_failed_ancestry_question_keeps_the_batch_and_a_clean_no_drops_it`
 and
`landing::service_land::finish_git_failure_tests::a_git_that_fails_does_not_make_a_landed_batch_look_unmoved`.

### `#4515` `first_commit_touching`, unborn-base arm

Not as filed:
`first_commit_touching` was deleted in `681791822c`
("the code left unreachable"),
 together with its controls.
The merge step now asks `git merge-tree --write-tree` for a tree and reports the conflicting
 paths and stages (`src/native/shadow/replay_merge.rs`),
 so no history listing runs and there is no unborn-base exclusion arm to cover.

### `#4516` `live_holder`, dead-holder arm

Not as filed:
`replay_reservation.rs` and its controls were deleted in `72e0534cca`.
The live-or-dead read it exposed survives as `classify_transaction_owner` and
`classify_transaction_owner_record` in `src/native/transaction/transaction_owner.rs`,
 whose dead arm is covered by
`transaction::transaction_owner::tests::dead_owners_are_dead`
 (a reused PID,
 an exited process,
 a zombie)
 and whose other-namespace arm is covered by
`an_owner_of_another_namespace_is_alive_until_its_record_expires`.

## `#6005` lossy path conversions against strict reads

The lease pair the issue names is gone:
`index_writer_coordination.rs` was deleted in `72e0534cca` and
`transaction_lease.rs` in `681791822c`,
 and `CLI_GIT_LANDING_LEASE` and `CLI_GIT_PREPARATION_LEASE` now appear only in three
 controls (`src/native/invocation/wrapped_command_tests.rs` and
`src/native/binary/binary_frontier_tests.rs`),
 never in production code.
The writing side is gone,
 so nothing writes a lease the strict read `lock_directory.to_str()` could miss.

The pattern itself survives once,
 in `landing/`,
 and it is left for the agent that owns that folder:

- writer:
  `src/native/landing/service_handoff.rs` writes
  `handoff.repository.display().to_string()` and `handoff.git_dir.display().to_string()`
  into a `WorkItem`,
  which loses every byte that is not UTF-8.
- reader:
  `is_acceptable` in
  `src/native/landing/service_work_handlers_replanning_and_acceptance.rs` compares
  `Path::new(item.git_dir.as_str()) == git_dir`.
- consequence:
  in a repository whose path is not valid UTF-8 the comparison cannot hold,
  and `src/native/landing/service_work_drain_pending_claims.rs` moves every work item for it
  to `FAILED`,
  so the push,
  review and upkeep never run.
- the idiom already in the tree,
  and the one to follow:
  `src/native/landing/service_claim_watch.rs` writes the path's bytes as hexadecimal under
  `FORMAT_PREFIX` and rebuilds an `OsString` from them,
  so any path round-trips.

## `#6005` fixed on `fix/git-policy-cli-service-work-items-keep-path-bytes`

The work-item pair above is fixed,
 and so is every other record in `src/native/` that kept a path as text and read it back as a path,
 compared it,
 hashed it,
 or passed it to Git.
The one idiom is `foundation/path_bytes_in_records.rs`:
 a field written twice,
 `gitDir` as plain text and `gitDirHex` as the path's bytes,
 with the bytes read first and the plain text kept for a record an earlier release wrote.
It is applied in `work_item.rs`,
 `push_retry_store.rs`,
 `service_scratch_ref_marker_of_a_batch.rs`,
 `merge_isolation_journal.rs`,
 `service_journal_intent_text_and_parse.rs`,
 `service_units_of_removed_worktrees.rs`,
 `worktree_removal_rescue.rs`,
 `service_protocol_encode_messages_to_lines.rs` and
 `service_protocol_decode_lines_to_messages.rs`.
`WorkItem::repository` and `WorkItem::git_dir` are `PathBuf` now,
 so no call site can compare text with a path again.

Still lossy,
 with the reason each is not fixed here:

- `src/native/worktree_copying/`:
  the four roots a journal names are `PathBuf` and each is written with a `Hex` field,
  read before the text,
  so recovery fills and reaps exactly the directory the record names;
  the stage claim carries `containerHex`,
  and the temporary of a write appends its suffix to the path's own bytes
  (`fix/git-policy-cli-worktree-copy-records-keep-path-bytes`).
  A repository path inside the copy
  (`selectedRoots`,
  `intendedEntries`,
  the created entries' `relativePath`,
  the install log's intents and the manifest)
  is still text decoded from Git's listing with replacement characters,
  so an ignored entry whose own name is not text is looked for under a name it does not have.
  That reaches the selection,
  manifest and install pipeline,
  so it is a job of its own.
- `src/native/repository_link_check/`:
  the gate holds `link_path`,
  `canonical_root` and `mount_point` as text and compares them with real paths;
  it is a report about links,
  not a step of a commit,
  so a lost byte misnames a finding there and nothing else.
- Messages and journal fields
  (`service_client.rs`,
  `service_commit.rs`,
  `push_trigger_log.rs`,
  `discard_audit.rs`,
  `worktree_slow_route.rs`,
  `worktree_dirty_restore.rs`,
  `real_git_resolution.rs`,
  and the `quote(...)` calls of `worktree_copying/` and `conflict/`):
  a byte lost in a line a person reads changes nothing.
- `src/native/landing/service_rules_precompile.rs` and
  `src/native/service/service_journal.rs`:
  each reads back what it wrote with the same lossy conversion,
  so the comparison stays consistent.
- `src/native/service/service_provision.rs`:
  the placeholders of a systemd unit file are text by the file's own format.
- `src/native/merge/merge_isolation_apply.rs` and
  `src/native/push/auto_push_reconcile_branch_advance.rs`:
  each names its root through the intent record above,
  which keeps the bytes.

One defect of another shape was found on the way:
 a commit's changed paths (`Change::path`) travel the socket as JSON strings,
 so a file whose name is not UTF-8 cannot be committed through the service at all.
That is a path inside a commit,
 not a repository path,
 and it needs the protocol's change list to carry bytes;
 it is not fixed here.

## What was run

- `mise run //package/git-policy/cli:native:test:host --` over the changed modules:
  102 passed,
  including the new `an_inherited_ignore_is_not_replaced_by_the_guard`,
  `sides_that_changed_different_paths_share_none` and
  `a_relative_hooks_path_in_a_bare_repository_resolves_against_the_git_directory`.
- The surviving controls named for `#4505` and `#4516`:
  `transaction::transaction_owner::tests::dead_owners_are_dead`
  (1 passed)
  and `landing::service_land::finish_git_failure_tests`
  (4 passed,
  including `a_failed_ancestry_question_keeps_the_batch_and_a_clean_no_drops_it`
  and `a_git_that_fails_does_not_make_a_landed_batch_look_unmoved`).
- Six guard removals,
  one filtered run each,
  to prove every new or changed control can fail;
  the failures are quoted in the commits.
- `node e2e/process-fixture.unit.test.ts` (this package),
  `node src/press.unit.test.ts` (`music-player/desktop-e2e`),
  `node src/typescript-sync-adapter.unit.test.ts`
  (`oxlint-plugin/prefer-readonly-parameter-type`).
- `mise run //package/git-policy/cli:native:clippy`:
  no finding.
- `mise run //package/git-policy/cli:lint:monochromatic-lint`:
  no finding.
- `mise run //package/git-policy/cli:lint:oxlint:paths -- e2e/process-fixture.unit.test.ts`:
  no finding.
- `lint:types` for the four packages whose TypeScript changed:
  all exit 0.
- `lint:oxlint` for `music-player/desktop-e2e`:
  no finding.
  For `oxlint-plugin/prefer-readonly-parameter-type` and
  `translation-repair/shared-contract` the packages report findings in files this branch does
  not touch (`task.ts` and `effect-default-library-reader-authority.ts`),
  so their suites were not read as green.
- `node src/bench-wire-model.unit.test.ts`
  (`translation-repair/shared-contract`) could not run in this worktree:
  the built entry point reaches `micromark-extension-math/lib/html.js`,
  which imports `katex`,
  and no `katex` exists in this worktree's `node_modules` or the main checkout's,
  even though `pnpm install --offline --frozen-lockfile` reports the lockfile up to date.
  The case changed there is an assertion on a fixture the file already built,
  and its type check passes.

## What this branch does not do

- It closes,
  comments on and edits no issue;
  that is the coordinator's step after review.
- It changes nothing under `merge/`,
  `conflict/`,
  `landing/`,
  `commit/commit_command.rs`,
  `push/`,
  `index/`,
  `worktree_copying/`,
  `worktree/`,
  `subvolume/`,
  `policy/` or `candidate/`,
  and nothing at a tokio call site.
- The `#6005` finding needs a change in `landing/`,
  which another agent holds.
