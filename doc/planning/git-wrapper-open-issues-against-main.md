# Open Git wrapper issues checked against `main`

What:
 the open GitHub issues that name the Git wrapper,
 checked against the code at
 `cf63cacad8`,
 and which of them still hold.

Why:
 several were fixed by work that never closed its issue,
 and the tree has moved under the rest,
 so the path an issue names is often gone while the defect lives on elsewhere.

## How the list was built

Three searches over the open issues,
de-duplicated by number,
gave 427 issues:

- `gh issue list --state open --limit 5000 --search 'git-policy in:title,body'` (330)
- `gh issue list --state open --limit 5000 --search 'cli-git in:title,body'` (388)
- `gh issue list --state open --limit 5000 --search 'git wrapper in:title,body'` (87)

Two mechanical passes ran over all 427:

- every backticked repository path in a title or body was tested for existence:
 208 of the 427 name at least one path that no longer exists.
- every code line quoted in an issue's fenced evidence block was searched for verbatim
 under `package/git-policy/cli/src`:
  228 issues quote code,
 and 79 of those still have their quoted line in the tree somewhere while 149 do not.

Those passes only say where to look.
Every issue classified in the sections below was then read in full and checked by reading the
 code it names,
 or by running the wrapper.
Issues whose named module is gone but whose behaviour is not were marked
 "not as filed" rather than fixed or closed.

## Fixes made on this branch

- `#4544` `fix(git-policy-cli): an attached -S key ID and -u mode stay one token` (`c8fa4f230e`,
 test `542ec5c6a1`).
 `normalise_inline_short` read the tail of `-S<keyid>` and `-u<mode>` as a cluster of flags,
 so `-S6ABCDEF` reached real git as `-S -6 -A -B -C DEF` and `-unormal` as `-u -n -o -r -m al`;
 git refuses the unknown switch,
  so a valid commit failed.
 Red before the fix,
 test `commit::commit_args::tests::normalise_keeps_attached_optional_values_whole`:
 `left: ["-S", "-6", "-A", "-B", "-C", "DEF"]`,
  `right: ["-S6ABCDEF"]`.
 Green after:
  that test and the other 25 `commit_args` controls.
 Real git accepts the attached spelling,
 measured in a disposable repository:
  `git commit --allow-empty -SZZZZ -m x` fails on the key,
 not on the spelling (`error: Couldn't load public key ZZZZ`).
 The incumbent TypeScript normaliser had the same defect.
- `#6110` `test(git-policy-cli): the empty-commit refusal is asserted beside the --allow-empty landing` (`9ffbb965e9`).
 The test claimed an empty commit lands only with `--allow-empty` and asserted only the with-flag side.
 The without-flag arm now runs the built wrapper and real Git over one repository,
 and pins the refusal,
  the shared status and the unchanged commit count.
 Green:
  `monochromatic-git-policy-cli::native_binary
 commit::an_amend_replaces_the_head_and_an_empty_commit_needs_its_flag`.

## Already fixed on `main`

- `#1757`:
  `914edf657c` (git-policy/cli) and `cde7ee971e` (monochromatic-lint) moved the tasks to
 `cargo nextest run`;
  `rg 'cargo test' package --glob 'mise.toml' --glob '*.mjs'` now finds no test invocation,
 so the ide and wg-quicker-exempt rows are covered too.
- `#1804`:
  `d926d5621a` added `src/native/push/push_stacking.rs`,
 and the commit subject says `Refs #1804`,
  which GitHub does not close on.
 A push that lost a race is merged and republished through the gates,
 and `src/native/push/push_stacking_tests.rs` plus `src/native/binary/binary_push_stacking_tests.rs` cover it.
- `#6261`:
  the crate's modules already sit in subsystem directories:
  33 folders under
 `package/git-policy/cli/src/native`,
  one `mod.rs` each.
- `#4504`:
  `adb421fabf` (`fix(git-policy-cli): keep an open batch when a Git question about its branch fails`).
 The old `recovery_completion.rs` and `commit_landed` are gone;
 the decision now lives in `src/native/landing/service_land_finish.rs`,
 where `landed_below` answers `Some(true)`,
  `Some(false)` or `None`,
 and only exit status 1 counts as "not below".
 Its own doc says "any other failure ... is no answer,
  and a batch is not dropped on no answer",
 and `recover_selected` stops on `None` instead of discarding the transaction.
- `#4545`:
  the code the issue names is gone.
 `src/native/commit/commit_conclusion.rs` is 64 lines with no `to_string_lossy`,
 and its module doc says the conclusion that once landed the commit is gone because the resident service is the
 only writer of a commit.
- `#575`:
  the guard exists and fired on this session:
 an `/usr/bin/git commit --message ...` in a disposable repository was blocked with
 "this command calls Git through an absolute path to the real Git binary,
  which skips the Git wrapper this
 repository installs on PATH".

## Claims that no longer hold

- `#6211`:
  `parse_lsof_number("")` returns `Some(0)`,
  and that is what the documented contract promises.
 `BigInt("")` is `0n` and `BigInt(" ")` is `0n`,
  measured with `node`:
  neither throws,
 so the incumbent at `f4fc1fa213:package/git-policy/cli/src/index-lock/index-lock-holders-darwin.ts:74`
 (which only catches what `BigInt` throws) also produced 0.
 The Rust and the incumbent agree,
 and `src/native/lock/index_lock_holders_tests.rs:33` pins it.
 A real divergence in the other direction could be filed instead:
  `BigInt("+5")` is `5n` while the port returns
 `None`.
- `#5974`:
  `management` holds no landing lock at all today.
 Neither `OwnerLock` nor `hold_landing_lock` appears anywhere under
 `package/git-policy/cli/src/native/invocation/`,
 and the comment the issue quotes is gone.
- `#2107`:
  `encode_last_pushed_record`,
  `auto_push_record.rs` and `LAST_PUSHED_SCHEMA_VERSION` are gone;
 `rg` finds no reference in the crate.
- `#4507`:
  `create_owned_file_link` is gone.
 `src/native/transaction/recovery_files.rs` is now five small operations,
 and each one that fails maps a non-`NotFound` error through `io_failure` with its operation and path,
 so no io reason is reported as an identity change.
- `#6007`:
  `parse_layout` and `replay_*` are gone.
 The layout report is produced in process by `repo_read_adapter::invocation_report`,
 whose path conversions are `to_str()?`,
 so a non-UTF-8 path gives no report instead of a lossy one.
 Whether a non-UTF-8 repository path should work at all is a separate decision,
  not this defect.
- `#6005`:
  `index_writer_coordination.rs` and `transaction_lease.rs` are gone,
 and no lossy-write against a strict `Path::to_str` compare survives in
 `src/native/lock/` or `src/native/landing/`.
 The lease inheritance code moved with the landing redesign,
 so this needs a fresh audit rather than a close.

## Still true, and left in another agent's folder

- `#4487` (folder `push/`):
  `run_git_bytes` still writes every byte of standard input before
 `wait_with_output` reads standard output,
 at `src/native/push/manual_push_range.rs:104` to `:108`,
 so a batch check whose output fills the pipe deadlocks the guard.
 This is the "nothing should ever wait forever" class and deserves the push agent's attention.
- `#4489` (folder `merge/`):
  `index_holds_the_merge` still looks each changed path up with a nested loop over the
 staged records,
  at `src/native/merge/merge_isolation_recovery_steps.rs:135` to `:140`.
- `#4490` (folder `merge/`):
  not verified beyond the surviving quote;
  `is_ignorable` still does not treat
 `--no-summary` as a `--no-stat` synonym.
- `#4537`,
  `#6006` (folder `worktree_copying/`):
  not verified.
- `#3513`,
  `#5971`,
  `#5972`,
  `#5976` (folder `push/`):
  not verified beyond the surviving quotes.

## Still true, and left for a follow-up (this crate, no decision needed)

- `#4576`:
  `duplicate_name` scans every earlier state for each manifest,
 `src/native/dependent_version/dependent_version_states.rs:197` to `:206`.
- `#4563`:
  `dependency_names` keeps source order with `names[start..].contains(&name)`,
 `src/native/dependent_version/dependent_version_manifest.rs:282`.
- `#6003`:
  `member_keys` rejects a repeated key with `keys.contains(&key)`,
 `src/native/config/config_values.rs:212`,
  and every object of the configuration document passes through it.
- `#2135`:
  four suites still hold `expect(true).toBe(true)`,
  one of them this crate's
 `package/git-policy/cli/e2e/process-fixture.unit.test.ts:36`.
- `#2108`:
  two `pub fn quote` remain,
 `src/native/foundation/json_record.rs:316` through serde and
 `src/native/dependent_version/dependent_version_fixture_json.rs:97` through UTF-16 units.
- `#4460`,
  `#4477`:
  the scanner,
  not the wrapper:
  `package/forbidden-strings/cli/src/walk.rs`.
- `#6187`:
  the scanner's test suite hardcodes `/usr/bin/git`,
 `package/forbidden-strings/cli/tests/ci_scan_scope.rs:33`.

## Needing a decision

- `#3024`:
  still true,
  and reproduced at the user boundary.
 In a disposable clone outside this repository,
 `monochromatic-git-policy-cli -C <clone> checkout -- tracked.txt` exits 1 with
 `linked-worktree-only/linked-worktree-required` and leaves the file untouched,
 so a foreign tool clone cannot run a guarded command.
 `src/native/invocation/effective_target.rs` exempts only the uv cache and the mise data directory,
 and a cargo build directory is not on that list,
 which is where `libghostty-vt-sys` keeps the vendored Ghostty clone the issue reports.
 Recommendation:
  add the cargo build directory (from `CARGO_BUILD_DIR`,
  else `CARGO_TARGET_DIR`) to
 `default_allowed_worktree_dirs`,
  beside uv and mise,
 and state the exemption's limit in that function's doc.
 This widens an exemption that protects data,
  so it needs the owner.
 The original incident's message text has changed with the redesign;
  the outcome has not.
- `#1768`:
  needs a decision,
  not a patch:
  no wrapper can notice its own absence.
 Recommendation:
  have the launcher that builds a child environment verify the resolved `git` and fail loudly or
 restore the directory,
  and keep a wrapper-side `doctor` check that answers whether policy is active for a
 repository.
- `#2108`:
  needs one sentence saying which side owns the contract;
 Recommendation:
  the fixture-side one,
  because Rust `&str` cannot hold a lone surrogate and `JSON.stringify`
 escapes one,
  and the release side keeps serde.
- `#1698`,
  `#1911`,
  `#3796` and the module-doc family below:
  one decision about how a stale citation is retired
 at scale,
  because each issue is a separate edit and they share one cause.
- `#428`,
  `#492`,
  `#574`,
  `#630`,
  `#358`:
  feature or scope questions,
  unchanged by this check.
 `#358` (publish the npm package) may be moot:
  the TypeScript wrapper and its package surface were deleted in
 `642b6d72cb`.

## Module-doc and decision-doc citations (one family, still true)

Around 230 of the 427 issues are citation reports:
a module doc,
decision record or test header names a file
that the native cutover deleted.
They are still true by construction,
and three samples were checked line by line:

- `#4500`:
  `src/native/policy/policy_root_context.rs:10` still names
 `package/git-policy/repository/src/index.ts`,
  which does not exist.
- `#4499`:
  `src/native/policy/policy_registry_tests.rs:45` to `:51` still lists nine removed TypeScript files as
 the sources of the registry defaults.
- `#4494`:
  `src/native/dependent_version/dependent_version_policy.rs:18` still cites
 `dependent-version-bump-policy.ts:178-235`.

They need the one decision named above:
 delete the citation and state the behaviour in the module's own words,
 or keep the citation with a `WR6` in-place move note.

## Paused by the owner

- `#1770`:
  the failure sits at the shadow and LFS-filter boundary,
  and LFS is paused.
- `#1332`,
  `#1797`,
  `#1798`,
  `#4103`,
  `#4586`:
  Windows and macOS,
  which the owner asked to be left alone.

## Not reached

Everything else among the 427 was sorted by the two mechanical passes but not read in full.
The groups,
with counts:

- citation reports other than the three sampled:
  the bulk of the 230,
 across cli-git decision records,
  troubleshooting records and other packages' docs.
- `refactor(*)` duplicate-code campaigns (`#611`,
  `#613`,
  `#614`,
  `#616`,
  `#617`,
  `#618`,
  `#619`,
  `#620`,
  `#621`,
 `#644`,
  `#646`,
  `#650`,
  `#651`,
  `#652`,
  `#655`,
  `#660`,
  `#671`,
  `#672`,
  `#679`,
  `#680`).
- untested-arm reports this crate owns and that were not verified:
 `#4505`,
  `#4514`,
  `#4515`,
  `#4516`,
  `#4530`,
  `#4585`,
  `#5972`.
 Their modules were rewritten (`recovery_contradiction.rs`,
  `replay_*.rs` are gone),
  so each needs a fresh read.
- `#2646`,
  `#2647`:
  auto-push,
  `push/`.
- `#573`,
  `#994`,
  `#6102`:
  logger sink-verification,
  Starship and timeout noise records,
 `package/module/logger` and its neighbours.
- the remaining service-lifecycle,
  worktree-removal and pre-commit-check findings,
 which three other audits are reading.

## What this branch does not do

- It closes,
  comments on and edits no issue;
  that is the coordinator's step,
  after review.
- It changes nothing in `merge/`,
  `conflict/`,
  `landing/`,
  `commit/commit_command.rs`,
  `push/`,
  `index/`,
 `worktree_copying/`,
  or at a tokio call site.
