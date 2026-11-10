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
- `#1135`,
  `#1136`,
  `#1137`,
  `#1138`,
  `#1139`,
  `#1140`:
  the cluster was filed against `src/native/real_index_lock_tests.rs` and its `real_index_lock` modules,
 which the lock redesign removed;
 `rg` finds the named test nowhere.
 The lock behaviour these report (a PID file beside a real index lock,
  a replaced lock released,
 a long name classified) lives in `src/native/lock/` and `src/native/transaction/recovery_files.rs`,
 so the six need a fresh read rather than a close.

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
 Three controls ran against the same debug binary:
 a clean linked worktree forwarded the same command and exited 0;
 a dirty linked worktree was refused with "rejected because this worktree has uncommitted changes to tracked
 files",
  which is the message the issue quotes;
 and the documented hatch,
  `restore --no-enforce-worktree`,
  exited 0.
 So the probe distinguishes the three cases,
  and the refusals are the policy's own.
 A build script cannot pass the hatch itself,
  so the build stays broken.
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

## What was run for these changes

- `mise run //package/git-policy/cli:native:test:host -- commit_args::`:
  26 passed.
- `mise run //package/git-policy/cli:native:test:binary:host -- an_amend_replaces`:
  1 passed.
- `mise run //package/git-policy/cli:native:clippy`:
  no finding.
- `mise run //package/git-policy/cli:lint:monochromatic-lint`:
  no finding.
- `mise run //:format:monochromatic-lint` and `mise run //:lint:monochromatic-lint` on this file:
  no finding.
- The whole host suite once:
 868 passed,
  4 aborted,
  3945 not run after the fail-fast stop.
 Every abort is the crate's own 120 second stuck-test watchdog (`RUN_LIMIT` in
 `src/native/testing/test_support.rs`),
 and the four tests are in files this branch does not touch:
 `config::config_fsync` (two),
  `conflict::discarded_prune` (one) and
 `foundation::earlier_log_lines_added_to_current_log` (one).
 They abort alone as well,
  on a host whose load average was 132.80 with 5228 tasks while three other sessions
 ran their own nextest jobs,
  and the wrapper's own log shows `git index-pack` making no sign of work for 50
 seconds.
 That is contention,
  measured rather than assumed,
  and it is not evidence about this branch;
 they need one run on an unloaded host or in the package's image.

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

## The duplicate-code campaigns, checked 2026-11-08T23:14:28-05:00 against `b942a2f9dd`

What:
 the open `refactor(*)` duplicate campaigns numbered between 611 and 680,
 plus #358,
#573,
#994 and #6102,
 each read in full and checked by reading the code it names.

Why:
 the earlier sections left these groups unread,
 and the tree has moved under both the paths and the quotes they carry.

Scope note:
 the tracker holds more than 5000 open issues,
 so `gh issue list --state open --limit 5000` returned no number between 611 and 680,
 and every number here was read directly with `gh issue view` and the GraphQL API.
 Issues #661,
#662,
#675,
#676,
#677 and #678 lie inside the same numbering block but are not
 duplication campaigns;
 #661 and #662 were read and are listed,
 #675,
#676,
#677 and #678 were not read.

Two refactors moved the ground under this whole group,
and they are the reason a missing path is not a missing defect:

- `592d78b143` dropped the `module`,
  `rust-module`,
  `dev-script`,
  `desktop-app` and other category
 directories,
 so `package/module/logger` is now `package/logger/core`,
 `package/dev-script/file-enforcer` is `package/file-enforcer/cli`,
 `package/cli/mvm` is `package/mvm/cli`,
 and the fork sidecars are `package/upstream-fork/<fork>.fuzz`.
- `642b6d72cb` deleted the TypeScript Git policy wrapper,
 so every `package/git-policy/cli/src/**/*.ts` citation in this group now names Rust under
 `src/native/`,
 and `e7b5130459` deleted the TypeScript file-enforcer with `file-enforcer.config.ts`,
 while `5bc05682ec` deleted the five linter packages `monochromatic-lint` replaces.

- `#611`:
 still true.
 `package/music-player/design/support.js` and `package/music-player/design/candidate/support.js`
 are still the same 69134 byte artifact,
 both md5 `951ae391b8ae72ef12e671c2fad23353`,
 and neither is tracked.
 The fix is still a page-specifier rewrite or a generated stub,
  so it is not small.
- `#613`:
 still true and wider than filed.
 `coverage-report.ts` now exists 13 times,
  for example
 `package/upstream-fork/p-map.fuzz/src/coverage-report.ts` and `package/logger/core.fuzz/src/coverage-report.ts`,
 and `fuzz-budget.ts` also exists 13 times rather than the three the issue measured.
 A shared package is the same size of change it was.
- `#614`:
 still true,
  smaller than filed.
 45 `.js` files still sit beside a `.ts` sibling under a `src/` folder,
 against the 120 the issue measured,
 for example `package/agent-harness-shared/session-discovery/src/index.js`.
 Naming and stopping the emitter is unchanged work.
- `#615`:
 still true.
 `package/music-player/design/_ds/material-design-3-design-system-503c3571-4db9-4b54-8695-7f53861f5059/_ds_bundle.js`
 is still present and no file under `package/music-player/design` outside the bundle mentions
 `_ds_bundle` or `ds-bundle`.
 Deleting it,
  or wiring it,
  is still the same decision.
- `#616`:
 still true.
 The vendored snapshot is at `package/upstream-fork/conf.fuzz/upstream-conf/index.ts`,
 both generated corpora are at
 `package/upstream-fork/deepmerge-ts.fuzz/src/type-soundness.generated.ts` and
 `declared-type-soundness.generated.ts`,
 and the archived scripts are still under `doc/handover/`.
 A marker-based classification step is a documentation and review change,
  not a code change.
- `#617`:
 still true.
 `function isAlphaNum` is still copied at `package/mvm/cli/src/config.ts:102`,
 `package/mvm/cli/src/backend/hetzner/ssh-key.ts:153`,
 `package/vmsync/cli/src/config.ts:102`,
 `package/claude-code-plugin/source/src/handler/bash-output-filter/validation.ts:32`
 and `package/agent-harness-shared/text-scan/src/characters.ts:73`.
 The consolidation is a 70 call site migration,
  so it is not small.
- `#618`:
 still true,
  one of its two defects gone.
 73 `isRecord` definitions exist and no `isObjectLike` family does;
 the array-accepting form is still at `package/wg/cli-quicker/src/tunnel-bypass-state-parse.ts:31`.
 The `pi-plugin/auto-mode` defect is gone:
 `7ac15b19b4` removed the legacy source and `package/pi-plugin/auto-mode/src` holds only `index.ts`.
- `#619`:
 still true.
 `pathExists` still reports absence for any `Error` at `package/mvm/cli/src/path-exists.ts:30`,
 `isErrnoException` still has 12 copies and `syncDirectory` three,
 and `package/fs-path/core/src/root-filesystem.node.ts:115` still exports the canonical probes.
 The three probe bugs are a few lines each;
 the consolidation is not.
- `#620`:
 still true.
 The wrong restore survives at `package/logger/core.fuzz/src/coverage-console.ts:124`,
 where `verboseArgv` pushes `--verbose` and pops it,
 which is wrong if anything else pushed an argument in between.
 That is a snapshot restore,
  so a few lines.
- `#621`:
 still true.
 Twelve `mise.verify-extension.ts` files exist,
  `fakePiApi` appears in six of them,
 and the throwaway home is still assigned rather than disposed at
 `package/search-fetch/pi-plugin/src/mise.verify-extension.ts:72`.
 The leaked home is a small fix;
  the shared harness is not.
- `#622`:
 still true.
 `fn reveal` is still duplicated at `package/file-manager/app/src/layout/scroll.rs:124` and
 `package/file-manager/variant-gtk-sticky/src/layout.rs:444`,
 and both GTK apps still carry parallel `pane.rs` and `strip.rs`.
 Settling the prototype's fate is a decision,
  not a patch.
- `#623`:
 still true.
 The decode cores are still separate,
 1818 lines at `package/music-player/android-app/rust/src/decode.rs` against 1588 at
 `package/music-player/desktop-app/src/decode.rs`,
 and `impl fmt::Display for PlayerError` is still written twice,
 at `android-app/rust/src/error.rs:299` and `desktop-app/src/error.rs:234`.
 Extraction is a crate move per stage,
  so not small.
- `#624`:
 still true,
  one half gone.
 `parseConfigJson` is still copied at `package/pi-plugin/guardrail/src/config-file.ts:126` and
 `package/search-fetch/core/src/config-file.ts:100`,
 with `readOptionalConfigJson` beside each.
 The `auto-mode` private model guard and comparator are gone with `auto-mode`'s legacy source.
 The config and message helpers are unchanged work.
- `#625`:
 still true.
 Silent catches survive at
 `package/claude-code-plugin/source/src/handler/claude-spawn/hook-session-start.ts:148`,
 `:180` and `:296`,
 and the pi side still has its own decomposed copy under `package/pi-plugin/spawn/src/state.ts`.
 Adding the logging is three small edits;
  the shared module is not.
- `#630`:
 still true.
 The shared base line is still generated independently at
 `package/git-policy/cli/e2e/scenario-subsumption-fixture.ts:126` and
 `scenario-shared-file-fixture.ts:214`,
 and the rename projections still disagree between
 `changeIds` at `commit-shape-trace-fixture.ts:605` and
 `selectedPaths` at `trace-replay-fixture.ts:243`.
 Deciding the rename order is small;
 sharing the arrangement is not.
- `#633`:
 gone.
 The file it names,
  `file-enforcer.config.ts`,
  was deleted in
 `e7b5130459 feat(*): delete the TypeScript file-enforcer and file-enforcer.config.ts`.
 What replaced it is `file-enforcer.config.jsonc` at the root,
 with the implementation under `package/file-enforcer/{cli,core,core-rust}`.
- `#634`:
 gone.
 The crate it asks `monochromatic-lint` to consume,
  `rust-linter-core`,
  was deleted in
 `5bc05682ec chore(*): delete the five linter packages monochromatic-lint replaces`,
 so there is no second span implementation to share with.
 `line_starts` at `package/linter/monochromatic-lint/src/rust_source.rs:50` is now that crate's
 only copy,
 and the `end_line()` defect is gone with it:
  no `end_line` or `end_column` exists under
 `package/linter/monochromatic-lint/src`.
- `#635`:
 gone.
 `package/linter/rust` was deleted in
 `5bc05682ec chore(*): delete the five linter packages monochromatic-lint replaces`
 and only its build directory remains on disk.
 Its rule loop,
  LSP framing and document sync were not carried into another crate.
- `#636`:
 still true.
 `function isAbortError` is still written twice,
 at `package/search-fetch/core/src/client-http.ts:341` and
 `package/search-fetch/core/src/exa-http.ts:324`,
 inside two complete JSON transports.
 The leaf helpers are a small move;
  the shared transport is not.
- `#637`:
 still true.
 The two Electron main processes are still separate files,
 `package/electron-infra/app-counter/src/main.ts` and
 `package/file-manager/variant-electron/src/main.ts`,
 and neither the lifecycle nor the Wayland switch list moved into `package/electron-infra`.
 Moving the Wayland switch list is small;
  the lifecycle framework is not.
- `#638`:
 still true.
 `togglePlay` at `1077`,
  `setPlayWhenReady` at `1147` and `playCurrent` at `1879` all live in
 `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlayerController.kt`,
 so the play-or-reload decision is still written twice.
 One `ensureLoadedAndPlay` is a small change.
- `#639`:
 still true.
 `renderOptionalComplement` is still copied at
 `package/i18n-compose/core/src/locale/ca/render-sentence-helpers.ts:83`,
 `locale/en/render-sentence-helpers.ts:86` and `locale/zh/render-sentence.ts:72`,
 and `nounPlural` is still copied at `locale/ca/render-np.ts:32` and `locale/en/render-np.ts:31`,
 where the Catalan fallback still appends an English `s`.
 The plural fallback is a small fix;
  the shared render layer is not.
- `#640`:
 mostly gone,
  one item left.
 `propertyValue`,
  `classifyArrayValue`,
  `defaultLibraryDeclaration` and `inspectActual` each exist
 once now,
 at `effect-result-reach.ts`,
  `readonly-collection-classification.ts`,
 `readonly-collection-suggestions.ts` and `effect-unresolved-capture.ts` under
 `package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/`,
 so the four divergences the issue names are gone.
 `oxlintOffset` is still copied three times,
  at
 `semantic-location.ts`,
  `overload-consistency.ts` and `readonly-suggestions.ts` in that folder,
 which is a small extraction.
- `#641`:
 still true.
 `unwrapExpression` is still three copies,
  at
 `package/oxlint-plugin/no-restricted-syntax/src/rule/no-array-callback-reference.arity.ts:62`,
 `no-array-callback-reference.ts:130` and `no-immediate-mutation.syntax.ts:108`;
 `isTsdocBlock` is still two,
  at `package/oxlint-plugin/stylistic/src/rule/require-asterisk-prefix.ts:89`
 and `package/oxlint-plugin/tsdoc/src/tsdoc-comments.ts:78`;
 and the static-name question now has four answers,
 at `rule/ast-shared.ts:64`,
  `prefer-error-is-error.syntax.ts:96`,
 `no-immediate-mutation.syntax.ts:139` and `arity.ts:92`.
 The `unwrapExpression` extraction is small;
  the rest is a migration.
- `#642`:
 still true.
 `attachedCommentsAt` at `package/toml-edit/core/src/comments.ts:27` and
 `attachedCommentValues` at `build-comments.ts:107` are still the same backward walk,
 `trailingCommentAt` at `comments.ts:98` and `trailingCommentValue` at `build-comments.ts:179`
 are still its trailing twin,
 and `skipTrivia` at `package/jsonc-edit/core-ts/src/parse-trivia.ts:36` and
 `captureTrailing` at `:144` still share the comment half.
 Sharing the comment scans is mechanical;
  the block matchers are not.
- `#643`:
 still true.
 Three runtime-directory helpers remain,
  at
 `package/wg/cli-quicker/src/opensnitch-operation.ts:80`,
 `tunnel-bypass-operation-lock.ts:18` and `tunnel-bypass-state.ts:150`,
 and the desired and installed identities still compute the same key,
 at `tunnel-bypass-route-owned.ts:114` and `:155`.
 Collapsing the two identities is a small fix;
 the Rust half is a separate change.
- `#644`:
 still true in part.
 The vm-builder copies survive:
  `run` at `package/vm-builder/cli/src/import.ts:98`,
 `build-and-import.ts:141` and `sign-and-push.ts:63`,
 and the chown drift is live,
  `process.env.USER ?? 'user'` at `import.ts:278` against the
 `CURRENT_USER` constant at `build-and-import.ts:114`.
 The glob,
  atomic-write and tracked-glob halves are gone with the TypeScript file-enforcer
 (`e7b5130459`).
 One owner rule for the chown is small.
- `#645`:
 still true.
 `isValidVmName` is character-identical at `package/mvm/cli/src/config.ts:145` and
 `package/vmsync/cli/src/config.ts:145`,
 and `validateName` at `:171` in both.
 `hetznerClone` at `package/mvm/cli/src/backend/hetzner/lifecycle.ts:428` still validates only the
 destination.
 Validating the source name is small;
 sharing the skeleton is not.
- `#646`:
 still true.
 `hasEnumerable` at `package/upstream-fork/deepmerge-ts.fuzz/src/model.ts:107` and `holds` at
 `options-model-merge.ts:76` are still two names for one enumerability test,
 `isPropertyKey` is still two spellings,
 and `describeSettlement` is character-identical at
 `package/upstream-fork/make-asynchronous.fuzz/src/settlement.ts:44` and
 `package/upstream-fork/pify.fuzz/src/settlement.ts:44`.
 Unifying the two enumerability predicates is small.
- `#647`:
 still true.
 `matching_close`,
  `split_top_level`,
  `group_inner_start` and `process_seq` are still copied into
 `package/forbidden-strings/core.bench/src/dialectport/atomwrap.rs`
 (`:170`,
  `:228`,
  `:211`,
  `:78`) from `src/port.rs` (`:328`,
  `:388`,
  `:370`,
  `:265`),
 and `fn db(peak: f64)` is still declared five times under
 `package/music-player/truepeak-core.bench/src/{buckets,classify,feasible,proportional,zoom}.rs`.
 Importing the four helpers is small.
- `#648`:
 still true.
 `jsonResponse` is still identical at `package/done/app/src/server/api/tasks.ts:52` and
 `timer.ts:38`,
 and `toSearchTask` is still defined twice in one file,
  at
 `package/done/app/src/lib/db/tasks-queries.ts:204` and `:232`.
 Deleting the duplicate mapper is small.
- `#649`:
 still true.
 `validate`,
  `deriveQuestionId` and `renderOption` are still parallel in
 `package/ssg/aquati.cat/src/component/question-checkbox.ts:162`,
  `:119`,
  `:203` and
 `question-radio.ts:160`,
  `:117`,
  `:201`,
 `langPage` at `src/page/lang.ts:34` and `tagPage` at `src/page/tag.ts:41` are still the same page,
 and `lightTokenDecls` at `src/style/tokens.ts:27` and `darkTokenDecls` at `:70` are still
 unenforced parallel maps.
 Merging the two token maps behind one table is small.
- `#650`:
 still true.
 `sumValues` is still two copies,
  at
 `package/git-clone-size/cli/src/combine.ts:65` and
 `package/monorepo-task/core-exec/src/depends-strategy.ts:58`,
 and `formatPosition` is still two,
  at
 `package/open-code-review-issue/cli/src/cli-output.ts:64` and `authority.ts:74`.
 Each item is a small move,
  which is why the issue asked for a per item decision.
- `#651`:
 still true.
 `flushOnExit` is still written twice,
  at `package/cloudflare-worker/rand/src/worker.ts:35` and
 `package/config/lfs-r2-worker/src/worker.ts:81`,
 and `which` is still written twice,
  at `package/terminal-exec/cli/src/validate.ts:86` and
 `windows.ts:48`.
 The PATH lookup is a small move.
- `#652`:
 still true in part.
 The JSONL parsers survive as separate policies,
  for example
 `parseRecord` returning `UNPARSABLE` at
 `package/claude-code-plugin/source/src/handler/stop-reminder/continuation-depth.ts:172`
 and `parseLine` at `package/git-policy/cli/e2e/jsonl-event-fixture.ts:93`.
 The Git LFS oid pair is no longer a pair of TypeScript files:
 the linter rule moved to Rust at `package/linter/monochromatic-lint/src/markdown_lfs_oid.rs`,
 so only `isOid` remains in TypeScript,
  at `package/config/lfs-r2-worker/src/oid.ts:39`.
 The oid half is gone;
  the JSONL half is unchanged work.
- `#653`:
 still true.
 `write` is still declared in four buffered sinks,
  at
 `package/logger/core/src/sink/indexed-db.ts:294`,
  `local-storage.ts:160`,
  `opfs.ts:182` and
 `session-storage.ts:106`,
 and `handOffBatch` and `flush` are still paired at `indexed-db.ts:272`,
  `:310` and
 `opfs.ts:159`,
  `:198`.
 One buffered-sink helper is a contained change.
- `#654`:
 still true.
 `fn event_time` is still two,
  at
 `package/nested-wayland-session/cli/src/input.rs:80` and `dnd.rs:462`;
 `fn rgba` is still two,
  at `package/ide/app/src/native/render.rs:34` and `annotate.rs:170`;
 the language-server executable probe is still two,
  at
 `package/ide/app/src/language/launch.rs:190` and `confine.rs:248`;
 and `SLINT_MCP_PORT` is still handled separately in
 `package/music-player/desktop-app/src/main.rs` and `package/terminal/app/src/main.rs`.
 The event clock and the executable probe are small each.
- `#655`:
 still true in part.
 The trusted-temporary-path pair it centres on is gone:
 `realpathOrUnavailable` and `tryRealpath` exist nowhere now,
 because `7ac15b19b4` removed `pi-plugin/auto-mode`'s legacy source and only `src/index.ts` remains.
 Still true are `envWrappedCursor` at
 `package/agent-harness-shared/terminal-title/src/command-wrappers.ts:48` beside
 `niceWrappedCursor` at `:88`,
 and the duplicated source-carrying visitors at
 `package/rolldown-plugin/import-attributes/src/scan-importer.ts:140` and `transform.ts:126`.
 Each survivor is a small extraction.
- `#660`:
 still true.
 `createGate` is still identical at
 `package/upstream-fork/p-limit/src/test-support.ts:49`,
 `package/upstream-fork/p-map/src/test-support.ts:50` and
 `package/upstream-fork/p-memoize/src/test-support.ts:83`,
 and the eight items are still undecided.
 The work is a decision per item,
  not a patch.
- `#661`:
 cannot tell.
 `package/file-manager/variant-qt/build.rs` is still present,
 but the warning is emitted by the third-party `qt-build-utils` build dependency at build time,
 so deciding whether it still fires needs a `lint:clippy` run of that crate on this host,
 which this audit did not run.
- `#662`:
 still true.
 `package/music-player/desktop-app/src/main.rs` still measures 296 code lines against the 300 limit,
 with the Slint allowance parked in `src/slint_generated.rs`,
 so the four lines of headroom the issue reports are exactly the headroom there is today.
 Extracting one cohesive block is a normal,
  contained refactor.
- `#664`:
 still true.
 `const evicted: string[] = [];` at
 `package/upstream-fork/quick-lru.fuzz/src/coverage-driver.ts:30` is still written to and then
 discarded by `void evicted;` at `:317`,
 and `deleteIfExpired` still appears eight times in
 `package/upstream-fork/quick-lru/src/quick-lru-iteration.ts`.
 Both are small,
  but each needs the decision the issue asks for.
- `#665`:
 still true.
 It is an index and names no code;
 its eight siblings `#666` to `#673` are all still open,
 so every pattern it indexes still lacks an owner.
- `#666`:
 still true and larger than filed.
 `this.name = ` appears at 407 sites and `extends Error` at 503 today,
 against the 210 and 241 the issue measured.
 The base class is a small package;
  the migration is 400 sites.
- `#667`:
 still true.
 `write_str(self.message.as_str())` still appears 11 times,
 and `String::from(message` now appears 24 times against the 17 measured.
 The macro crate is small;
  the crate named in the issue,
  `rust-module/error-support`,
  does not exist.
- `#668`:
 still true and larger than filed.
 `io::Error::new(io::ErrorKind::` appears at 42 sites today against 15,
 24 of them under `package/wg/cli-quicker-exempt`,
 so the package-local helper is still the right first move and still small.
- `#669`:
 still true,
  unchanged.
 22 `attachShadow(` sites against 25 `extends HTMLElement` declarations,
 still waiting on the `done-postcss` decision.
- `#670`:
 still true and slightly larger.
 `Object.create(null` appears at 24 sites today against 21.
 The helper and its test are small.
- `#671`:
 still true and much larger.
 `.localeCompare(` appears at 67 sites today against 40.
 The combinator is small per site,
  and the issue's own sequencing still applies.
- `#672`:
 cannot tell,
  and the pair it names is now a single site.
 `stripAttrQuery` survives at
 `package/rolldown-plugin/import-attributes/src/patterns.ts`,
 but `modulePath` went with the TypeScript Git policy wrapper in `642b6d72cb`,
 and no `module_path` exists under `package/git-policy/cli/src/native`.
 Deciding this needs the owner's answer on whether one site plus a convention is worth a package.
- `#673`:
 still true.
 The batch wrappers are still eight hand-written bodies in
 `package/forbidden-strings/core/src/regex/batch.rs`,
 for example `is_match_batch_scalar` at `:221`,
  `is_match_batch_interleaved` at `:247`,
 `batch_inter_w` at `:296`,
  `batch_tight_w` at `:323`,
  `batch_sheng` at `:351` and
 `batch_sheng2` at `:378`,
 with `is_match_batch_bucketed` at `:165` correctly left out of the family.
 The macro is a contained change.
- `#679`:
 still true,
  with the one red site it named gone.
 `package/git-policy/cli/bin/build-git-test-image.mjs` became `build-git-test-image.ts`
 and carries no `RegExp`,
  so the reached-and-red site is gone,
 but the coverage gap stands:
  43 of the 271 package `mise.toml` files mention no `lint:oxlint` task at all,
 so the fanout in root `mise.toml` still cannot reach them.
 Adding the task and covering `doc/**` is the same work it was.
- `#680`:
 still true as an index.
 `doc/audit/regex-necessity.md` and the harness under `doc/audit/regex-necessity/`
 are still in the tree,
 and it names no source of its own to check.
- `#358`:
 gone.
 The package it asks to publish,
  `@monochromatic-dev/cli-git`,
  no longer exists.
 `package/git-policy/cli/package.json` now names `@monochromatic-dev/git-policy-cli`
 and describes itself as the verification harness of the Rust crate,
 so publishing it would publish a different thing than the issue audited.
- `#573`:
 still true.
 The message still comes from the buffered sinks,
  for example
 `sink verification failed` in `package/logger/core/src/sink/session-storage.ts`,
 `local-storage.ts`,
  `opfs.ts` and `indexed-db.ts`,
 and the logger's own `DECISIONS.md` records the verification step.
 Finding which process logs it needs the failing run,
  not more code reading.
- `#994`:
 still true.
 `doc/troubleshooting/starship-noninteractive-command-noise.md` now records the local `.bashrc`
 cause and the stderr-muting workaround,
 and states that no prototype exists and that the user asked for a stopgap,
 so the durable fix the issue asks for has not been made.
- `#6102`:
 still true in part.
 A note naming `642b6d72cb` was added beside the first excerpt,
 but `doc/troubleshooting/starship-cli-git-command-timeout.md` still says
 "The same identity reaches final forwarding at" the deleted
 `package/git-policy/cli/src/bin.ts:338-348`,
 and still names the deleted
 `package/git-policy/cli/src/read-only-forwarding.unit.test.ts` as the regression fixture.
 Relabelling two excerpts and naming the Rust test that inherited the guard is a small edit.

## What was run for this section

- `gh issue list --state open --limit 5000 --json number,title`:
 5000 issues,
  none numbered between 611 and 680.
- `gh api graphql` with one alias per number for all 74 numbers in 611 to 680 plus 358,
  573,
  994
 and 6102:
 59 open in that block,
  plus the four named ones.
- Every open body read in full.
- Every "still true" line was checked with `rg` or `read` at the cited path on this branch at
 `b942a2f9dd`.
- `mise run //:format:monochromatic-lint -- doc/planning/git-wrapper-open-issues-against-main.md`:
 reformatted the file as shown here.
- `mise run //:lint:monochromatic-lint -- doc/planning/git-wrapper-open-issues-against-main.md`:
 no finding.

## What this section does not do

- It closes,
  comments on and edits no issue.
- It changes no source file.
- It does not read #675,
  #676,
  #677 or #678,
 which sit in the same numbering block but are not duplication campaigns,
 and it does not re-measure the drifts inside a campaign beyond the code the issue names.
