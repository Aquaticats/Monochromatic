# What `cargo update` does under the cargo shadow

Status:
decided by the owner on 2026-11-03 ("I said follow pnpm semantics").
The earlier options (pass through,
shadow-owned update,
run-then-check with rollback,
refuse and redirect) are superseded;
 the cross-vendor review of them,
if it returned,
is in the commit history of this file.

The rule:
`cargo update` under the shadow behaves like `pnpm update`,
pnpm's own policy-aware command
 ([pnpm update][pnpm-update],
read 2026-11-03),
and `cargo generate-lockfile` like `pnpm install --lockfile-only`
 ([pnpm install][pnpm-install]).
Cargo's resolver never chooses freely:
the release-age and trust policy of `rust-catalog.toml` applies while versions are chosen.

The flag mapping lives in `package/cargo-shadow/cli/src/update_plan.rs` and the README of the package.
pnpm has no counterpart for `--breaking` as a cargo-resolved update,
and none for a dry run of `update`;
 the nearest pnpm behavior was taken for each (README,
"Mapping `cargo update` to `pnpm update`").

[pnpm-update]: https://pnpm.io/cli/update
[pnpm-install]: https://pnpm.io/cli/install

## Cross-vendor review of the earlier options

A reviewer from another model vendor (`openai/gpt-6.1-sol`,
no tools,
only the first version of this file) answered on 2026-11-03.
It had no repository access,
so its points are about design,
not about this code.

- It preferred,
  over any live-file rollback,
  running the native operation against a private candidate lockfile,
   validating that candidate,
  and publishing it only if validation passes and the original lockfile has not changed meanwhile.
  It called this feasibility-dependent:
  the toolchain must be able to target a private lockfile,
  copying a workspace breaks path
   dependencies and configuration discovery,
  and a content comparison followed by a rename is not an atomic compare-and-swap.
- Against snapshot and restore,
  which is what the update does today:
  - a concurrent writer between the snapshot and the restore loses its change (restoring is atomic,
    overwriting someone else's work is not);
  - a rejected version is visible to other readers while the check runs;
  - a process killed after cargo wrote the lockfile and before the restore leaves the unchecked lockfile behind;
  - restoring must restore absence,
    not an empty file (the shadow's full update removes the stand-in lockfile it created on a refusal).
- A version that passes the release-age and trust policy can still fail to build;
  the policy is not a compatibility check.
- A "trust policy" with pnpm's name is not automatically the same rule as the crate catalog's;
  the catalog's levels are heuristics from crates.io data.
- It could not verify pnpm's write ordering for `pnpm update`;
  the owner's answer settled the question by rule instead.

The package README states the transaction limits.
A staged candidate lockfile (`cargo -Z unstable-options --lockfile-path`,
if the pinned toolchain has it) is the open improvement.

## Decisions of 2026-11-03 and what each rests on

Each follows from words the owner said;
pnpm references are to the sources of pnpm 12.16.1 (the installed version,
 cloned at tag `v12.16.1`,
commit `c7b46ae`).

- The package exists,
  and the Rust catalog tools live in it:
  "Make a cargo shadow like what we did for pnpm shadow.
  Put the existing rust catalog tools inside it instead of making them separate."
- `cargo update` is `pnpm update`,
  `cargo generate-lockfile` is `pnpm install --lockfile-only`:
  "I said follow pnpm semantics."
  The flag mapping is in `package/cargo-shadow/cli/src/update_plan.rs` and the README.
- The mise tasks `rust:install`,
  `rust:update` and `rust:verify` are removed:
  "I also said to remove the Mise tasks that would then be handled via the wrapper."
  Workflows never use mise ("Workflows should never use mise at all."),
  so `advisory-core.yml` calls `cargo-shadow` directly.
- An operational failure of the install step (no network,
  a registry lookup that fails,
  a cargo error) warns and runs the
  command anyway,
  with `--locked` added once,
  before any `--`.
  pnpm:
  "If that install fails,
  for example because the store is read-only or there is no network,
  pnpm prints a warning and runs
   the command anyway" (<https://pnpm.io/settings/build>).
  pnpm can do that because `pnpm run` never rewrites the lockfile;
  cargo does,
  hence `--locked`.
  A frozen check (CI,
  or the user's own `--locked`) that finds a stale lockfile keeps this behavior too:
  "CI keeps the frozen
  behavior already recorded (the 2026-10-29 trustLockfile amendment)".
  A frozen policy refusal stops the command by a delegated
  pick of 2026-11-03 (below).
  Replaces the fail-closed behavior built first,
  and,
  for a policy refusal of a local install,
  is replaced in turn by the next two decisions.
- A local (not frozen) install sets a release under `minimumReleaseAge` aside instead of refusing it.
  The owner,
  2026-11-03 (question tool,
  verbatim):
  "Try asking better.
  pnpm nowadays just warn and runs when a dep could be updated to something published within a day."
  pnpm 12.16.1 resolves to the newest release older than the cutoff
  (`resolving/npm-resolver/src/pickMatchingVersion.ts`,
  `pickRespectingMinReleaseAge`)
  and the coordinator found no message printed for it in the source.
  The install step that runs before a resolving cargo command therefore moves a version it newly needs,
  and that is under the
  age limit,
  to the newest older release in range that passes the policy (the set-aside `cargo update` already does),
  prints one line naming the held-back release and the one it used,
  and runs cargo.
  Only when no release in range passes does the refusal stand (`minimumReleaseAgeStrict = true`,
  as pnpm's strict mode).
  The owner can still veto this.
- A committed lockfile that holds a refused release is changed to an older one on a local run,
  and the command stops only when none fits.
  Both answers are "Set it back,
  build (Recommended)" (question tool,
  2026-11-03).
  Case 1,
  release age only:
  a merge brings a Cargo.lock holding serde 1.0.230,
  published 3 hours ago;
  the limit is one day;
  1.0.229 is nine days old and fits the range;
  the person runs `cargo build` locally.
  Answer:
  the shadow changes serde in Cargo.lock from 1.0.230 to 1.0.229,
  prints one line naming both,
  and builds.
  If no older release fits the range,
  it stops.
  Case 2,
  trust downgrade:
  Cargo.lock holds foo 2.3.1,
  from an account with no earlier releases,
  while 2.3.0 came from the crate's trusted CI publisher.
  Answer:
  the shadow changes foo to 2.3.0 in Cargo.lock,
  prints one line naming the publisher change,
  and builds.
  If no earlier release with the earlier trust level fits the range,
  it stops.
  Both rest on pnpm's effect:
  the lockfile check runs before any download,
  the refused release never runs,
  and the script runs on the older copy.
  The questions shown to the owner were scoped to local runs,
  so CI keeps the frozen behavior already recorded.
  An operational failure keeps warn-and-run-with-`--locked`.
  Which entries are checked follows `trustLockfile`:
  all of them when false (pnpm's default),
  the ones new since the base revision when true (the repository's setting).
  The set-aside goes through the same transaction and ownership rules as the review findings below,
  so it never overwrites
  another writer's lockfile.
- The crates.io database dump path is ported (`dump.rs`,
  `csv.rs`),
  because the owner moved the whole tool into the shadow;
  a cold run reads one streaming pass instead of one API request per crate at 1.1 seconds.
- `--breaking` raises the catalog floors and runs the sync,
  as `pnpm update --latest` writes the new range to the catalog entry:
  `pnpm11/installing/deps-resolver/src/index.ts:280-322` (`getUpdatedCatalogs`,
  `getCatalogUpdatesOfProject`) collects,
  for each
   wanted dependency with a catalog lookup and `updateSpec`,
  the updated range;
  `wantedDepShouldUpdateCatalog.ts:3-5` makes that
   true unless `saveSpec` is false (`--no-save`).
  The project manifest keeps `catalog:`;
  the catalog file gets the range.
- A frozen install verifies per `trustLockfile`,
  and there is no `verify` command (correction of the first reading,
  which verified
   every locked entry and contradicted the owner's decision of 2026-10-29 in `doc/decision/rust-dependency-catalog.md`:
  "the CI
   check covers only registry versions that no `Cargo.lock` held at a base revision ... the counterpart of pnpm's
   `trustLockfile: true`").
  pnpm:
  `pnpm11/installing/deps-installer/src/install/mutateModules.ts:314-343` (`startLockfileVerification`) re-validates every entry
   of the lockfile against the resolver policies on install,
  skipped only when `opts.trustLockfile` is set;
   `verifyLockfileResolutionsCache.ts` caches a passing lockfile by hash;
  `pnpm11/workspace/state/src/validation.ts:428` has
   `trustLockfile: false` as the default;
  `pnpm11/installing/deps-installer/test/install/trustLockfile.ts:21-52` shows a frozen
   install rejecting a refused entry and `trustLockfile: true` skipping it.
  Here `trustLockfile` is a catalog setting (`package/dependency-catalog/core-native`,
  0.2.0,
  and the TypeScript reader
   `core-rust`),
  true in `rust-catalog.toml`.
  True:
  only versions new since the base revision (`CARGO_SHADOW_TRUST_BASE`,
  `HEAD` by default;
  CI sets the pushed range's previous
   commit or the pull request's base),
  checked for trust.
  False:
  every locked entry,
  remembered by lockfile digest.
  A non-frozen install and `cargo update` check what they newly resolve in both cases.
- The bin name stays `cargo-shadow`,
  matching `pnpm-shadow` and `mpm-shadow`;
  cargo also runs it as `cargo shadow`.
- The toolchain `toolchain.lock.json` pins is the one the install step runs,
  so one cargo writes every lockfile.
- Lockfile writes never discard another writer's data ("never discard user data";
  "Use as few locks as possible."):
  no lock is taken;
  a restore replaces a lockfile only when it still holds what the run produced,
  by atomic rename,
  and a journal
   lets the next run finish a killed one (`journal.rs`).
  `cargo -Zunstable-options update --lockfile-path` does not exist on the pinned nightly (`cargo update --help`,
  2026-11-03),
  so
   resolving into a private lockfile was not possible.
- The launcher test is bounded by a nesting counter in the environment and a 20-second deadline.
  With the spawn guard removed from `cargo_run.rs` it fails within seconds (GFP).

## Review findings fixed (cross-vendor review of 2026-11-03T13:26:08Z)

The review ("Do not rely on this implementation as a policy-enforcement boundary yet.")
was a static reading of the diff;
each finding was reproduced with a failing test before it was fixed (the commits are on `feat/cargo-shadow-cli`).
What changed,
and what each rests on:

- Linux only:
  the owner's "Ignore Win and mac until I say so."
  The journal's `/proc` liveness probe is valid only there,
  so the crate refuses to compile elsewhere (`lib.rs`).
- Refused lockfile copies get collision-free names,
  recovery errors keep the journal,
  and unknown ownership never authorizes a restore:
  "never discard user data" and "Fix even pre-existing issues of the pkgs you own."
  Each lockfile a run writes is recorded as the run's own output right after its cargo call returned,
  and a restore replaces only that.
- Unknown or malformed timestamps never grant an age exemption (`trustPolicyIgnoreAfter`),
  and the owner list is asked whenever it can change a trust verdict.
- Unsupported registries are refused by name,
  because registry identity is not carried end to end.
- The freshness stamp certifies manifest consistency only;
  the policy check has its own remembered pass keyed by lockfile text,
  settings,
  mode and base spelling.
  Measured cost of the fast path was not repeated after this change.
- A local install checks existing entries (pnpm's `startLockfileVerification` is not gated on frozen).
- `--breaking` edits the manifests,
  so they are part of the transaction.
  The catalog write and the `file-enforcer` sync are inside the transaction by a delegated pick of 2026-11-03 (below).
- Update arguments are read in every supported spelling,
  before and after the command word,
  and an unmapped option is refused.
- The CSV reader refuses malformed tables and bounds fields,
  rows and histories.
- A full update whose strict cutoff cannot resolve a crate that `minimumReleaseAgeExclude` exempts resolves without the cutoff and
  then holds the other crates to it.
  This is the one shape cargo's `--publish-time` (no per-crate form) allows,
  and it is open to the owner's veto.
- Set-aside candidates are bounded by what cargo accepts for the manifests,
  not by the caret class,
  honor the exemptions and the strictness,
  and are judged with the owner list.

## Review findings fixed (second cross-vendor review, started 2026-11-03T16:25:50.931Z)

The second review covered `504a82aa3f..09bb7820ea` in three parts (Git reader, journal and restore, install and policy).
The nine delegated picks below stand;
the review found defects in how they were built, not in the picks.
What changed:

- Ownership of an output file is decided in one place (`file_owner.rs`):
  the kernel's inode change time against the window of the cargo call, never by reading the file after the child returned.
  A file that differs from its snapshot and changed inside the window is the run's output;
  one changed later is another writer's and is never restored over.
  The remaining blind spot (a writer that publishes within about one tick of the call, or during it) is documented;
  the restore keeps every file it displaces, so it loses no data.
- The replacement protocol is a preservation contract (`journal_replace.rs`), not an atomic conditional replace:
  exclusive, synced temporary files;
  `RENAME_EXCHANGE`;
  the displaced file is read through its descriptor and hard-linked to a surviving name before anything is unlinked;
  an unexpected file is exchanged back and an edit made after the exchange is kept;
  a filesystem without exchange fails visibly;
  directories are synced.
- The journal is durable and claimable:
  snapshots are synced and the index is the commit record;
  recovery claims a journal by atomic rename and decides per root;
  the owner is a pid with its process start time;
  the manifests of a killed `--breaking` run are journaled in a lossless, versioned record.
- A failed cargo call that wrote a lockfile is rolled back like a successful one;
  a rollback that cannot complete stops the command (`InstallError::Unsettled`) instead of letting `--locked` build it;
  an unsupported registry is a policy refusal (`InstallError::Refused`) in every path.
- A lookup failure after a release-age refusal keeps the refusal (`check_policy`);
  frozen verification reports the non-strict age warnings;
  a malformed historical timestamp establishes no publisher history;
  `cargo update --precise` judges the release it names even when it was already locked (`precise_target.rs`).
- The placeholder lockfile is taken by rename before it is read;
  a failing floor sync restores the catalog only while it holds the raised text and records the manifests it rewrote.
- Update options need their values, `-m<path>` is understood, and the command word is found by position.
- A CSV row has a cumulative byte budget besides the field and field-count bounds.
- The Git reader follows the lexer of Git's `config.c`, reads the first `merge` value, refuses any `extensions.*` variable,
  worktree-private references, invalid or unusual refspecs and objects it cannot see to be commits,
  and asks Git once for every candidate when it declines.
  It is tested against git itself (`git_differential_tests.rs`).
- The system configuration path of the Git that answers is assumed to be `/etc/gitconfig`;
  finding it needs a git start, so this is a stated limit and not fixed.
  Power-loss durability is not tested with fault injection.

## Review findings of the third cross-vendor review (started 2026-11-03T17:33:52Z)

The third review found the replace protocol and the journal still failing on the same schedules, so the protocol changed instead of
getting another comparison.
Where an earlier section of this record says a finding was "fixed" and this section says otherwise, this section is current.

- Ownership is no longer what protects data.
  A restore never unlinks an inode that was live at a path (the displaced file and the replacement each get a surviving name first),
  a file that vanished stays vanished, and `ctime` only chooses between "restore and keep the displaced file" and "leave alone".
  Misattribution moves a foreign file aside with a warning, or leaves cargo's output live, which stops the command (`Unsettled`).
  This is the design the coordinator preferred on 2026-11-03: correctness that does not depend on attribution.
- Liveness is a lock, not a guess (delegated pick, open to the owner's veto).
  Every run holds one exclusive `flock` on a file in its own uniquely named journal; a recovery tries for it without waiting.
  The owner's rule "Use as few locks as possible." allows one per run; it replaces the process-id directory name, the `/proc` start
  time and the claim rename, which the review showed fail open (reuse, unreadable `/proc`, directory-name reuse, overlapping live runs).
  A recovery skips roots a live journal covers.
  A `ready` record, written last, lists every journal file with its length.
- Durability: file data before it is relied on, each new directory with its parent, kept outputs before the journal goes, a `.done`
  marker before removal.
  There is still no fault-injection test, so no power-loss guarantee is claimed.
- The owner's rule of 2026-11-03, "nothing should ever wait forever or indefinitely", is applied to every wait: locks are tried,
  child processes run under bounds that end in an error naming the call and the command to run again (cargo 30 minutes, Git 60
  seconds, `file-enforcer write` 5 minutes, a pause for HTTP 429 at most 2 minutes, the dump download an hour), with every file kept.
  These are chosen limits, not measurements, and they are delegated picks open to veto.
  The one exemption is the user's own command after the checks.
- An update verifies the lockfiles it leaves the way an install does (every locked version, or every version the base did not hold
  under `trustLockfile`); a rollback that cannot complete stops the command even when the policy report is the reason;
  the stand-in lockfile is kept before its name is removed; an owners lookup failure leaves a provisional finding unreported;
  a local pass with a warning is not remembered; a recording error around the sync still puts the catalog back.
- The Git reader sees symbolic-link references, the remote `.` mapping, exact path text, whole objects and all remote configuration,
  declines for repositories another user owns and for system configurations in other known places, and every Git call is bounded
  and reaped.
  Not fixed: a system configuration in a place no list knows of.
- `cargo add` and `cargo remove` are an explicit exclusion: they run cargo first and only warn; the policy is enforced by the install
  step of the next command that resolves dependencies, and only if that command goes through the shim. Dispatch tests for the
  whole table of verbs are not written (round 6: open).

## Review findings of the fourth cross-vendor review (started 2026-11-03T18:56:11Z)

- One workspace lock replaces the per-journal lock (delegated pick, open to the owner's veto; "Use as few locks as possible.").
  A transaction and a recovery both take it without waiting, so a recovery cannot race a transaction that starts, and a
  transaction that finds it held stops with the holder's process id.
  The lock is close-on-exec, because an inherited descriptor kept by a daemon that cargo starts would make the workspace
  unrecoverable indefinitely; the children of a killed wrapper are found by the process groups recorded in the journal instead.
- Every child runs in its own process group; at a bound the group is killed and shown to be gone, a normal exit kills what is left in
  the group, and output is returned only when complete.
- Finishing a run forces its outputs to disk before the completion marker (the owner's durability answer "Force at leaving, else
  background"); the background half (syncing what a command did not promise) is not built.
- Kept files are never deleted or refused over by the tool (owner answer of round 6, "Warn past a count, never refuse (Recommended)"):
  past 1,000 files every run prints the count, the total size and the folder; the 1,000 is a warning point open to veto.
- The system configuration of the Git reader is what the answering Git reports; the answer is reused only for a compiled program
  (round 6 replaced the wrapper-blind cache key).
- Limits are measured or labeled unmeasured beside their values (repository rule LQM); the numbers measured on 2026-11-04 are in
  the README.
- pnpm oracle (pnpm 12.16.1, `minimumReleaseAge: 5256000`, a lockfile pinning `is-odd@3.0.1`, whose dependency `is-number@6.0.0` is
  also younger than the cutoff): `pnpm install --lockfile-only` fails with `ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION`; `pnpm update`
  fails the same way (and, when the policy is not strict, first adds the entries to `minimumReleaseAgeExclude`); `pnpm update
  is-odd` fails with `ERR_PNPM_NO_MATURE_MATCHING_VERSION` when strict and with the same violation for `is-number@6.0.0` when not.
  So pnpm, too, refuses an unchanged full update that leaves a too-young version locked in this configuration.
  Round 6 narrows the claim: the oracle shows that for a full update and for a target whose own dependency is too young.
  It does not show what pnpm does for an unrelated refused package outside a targeted update's dependency closure (cargo-shadow
  verifies the whole final lockfile, which can be stricter), for ordinary, latest and precise selection, for a version trusted at the
  configured base, or across unrelated workspace roots. Those cases are open, and the difference is a candidate for the owner's veto.
  Another difference: pnpm's non-strict mode edits `minimumReleaseAgeExclude` as a side effect, and cargo-shadow does not.

## Delegated picks of 2026-11-03

The owner is away for about 8 hours and delegated the open questions of the review work:
"Pick options as if I had picked them and invite veto when I say I'm back."
These are the coordinator's picks under that delegation,
not owner answers,
and the owner's veto is open when they are back.

- A local run compares lockfiles with the upstream branch (`@{upstream}`,
  falling back to `origin/main`,
  `origin/master`,
  `HEAD`)
  under `trustLockfile = true`,
  so a committed but unpushed merge is new since the base and cases 1 and 2 fire locally.
  CI keeps its base (`HEAD`,
  or `CARGO_SHADOW_TRUST_BASE`,
  which wins everywhere).
  A remembered pass is keyed by the resolved commit,
  not by how the base is spelled.
  The base is read from the repository's files (`git_refs.rs`,
  `git_config.rs`,
  `git_native.rs`) without starting git,
  by the owner's rules
  "Invoke real git as rarely as possible."
  (2026-11-01T01:28:19-05:00,
  quoted in `package/git-policy/cli/SPEC.md`) and "embed instead of spawning";
  git is started once for an explicit base,
  a configuration include,
  reftable,
  a worktree configuration,
  a `GIT_*` variable that moves the
  repository,
  a negative refspec,
  a symbolic reference or a shadowed short name (each tested).
  The first version of this pick spent a median of
  7 ms on a git start per call;
  the measured saving of the file reader is in the README.
  The in-repo code that reads these natively (`package/git-policy/cli`) is a binary-oriented crate with its own dependency tree;
  depending on it from this crate would be a new dependency,
  so it was not added and its approach was reimplemented for the few keys needed.
- The release age and the trust policy both apply to versions new since the base,
  on the local and the frozen path.
- A policy refusal stops the command on the frozen path too (pnpm's frozen install fails on a rejected lockfile entry;
  `verifyLockfileResolutions.ts` throws before any fetch).
  A stale lockfile and operational failures keep warn-and-`--locked`.
- The window in which a cargo that is not the shadow builds on an unapproved live lockfile is accepted and documented
  (owner rule "Use as few locks as possible").
- A restore compares and replaces with `renameat2` and `RENAME_EXCHANGE` through `rustix`'s safe wrapper,
  and undoes the exchange
  when the file it displaced is not the expected one.
  `rustix` was added to `rust-catalog.toml` (`>=1.1.6`,
  the version tempfile already
  locks) and to the crate's `Cargo.toml`.
  The installed `file-enforcer` 0.1.1 binary rejects `trustLockfile` in this catalog,
  but the
  repository's source (`package/file-enforcer/cli`,
  which depends on `monochromatic-dependency-catalog` 0.2.0) accepts it:
  built through `mise run //package/file-enforcer/cli:build:debug`,
  `file-enforcer write` regenerated the
  `[workspace.dependencies]` line byte for byte as the first hand-written version had it (no diff),
  and `check` is clean.
  The installed binary is stale,
  not the source.
- A strict cutoff with exemptions keeps the approach recorded above:
  resolve without the cutoff,
  enforce it on the other crates.
- A `file-enforcer` sync failure after the catalog write of `--breaking` restores the catalog (and the update is not kept).
- A killed `--breaking` run is recovered for the manifests,
  which are journaled with the lockfiles.
- The trust-downgrade finding names the new publisher and the earlier release with its publisher (the account id,
  or the
  trusted-publishing repository) next to the trust levels;
  the registry reports ids,
  not account names.

Still open:
nothing from the earlier list;
the cost of the git start on every call is the one thing to look at again.

## Review findings of the fifth cross-vendor review (started 2026-11-04T01:50:38Z)

Owner answer, round 6: "Warn past a count, never refuse (Recommended)".
Past 1,000 kept files every run prints the count, the total size and the folder; nothing is refused or deleted.
The 1,000 is a warning point carried over from round 5 and open to veto; the rate at which files are kept was not measured.

Fixed, each with a red test committed before the fix:

- Kept folder: a notice instead of a refusal (`a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses`),
  and an unreadable folder is reported instead of read as small.
  The notice counts the kept folder only; files kept beside their target and stranded temporaries are reported by the recovery
  (`report_leftover_temporaries`), so a reservation of capacity before a transaction is overtaken by the owner's answer.
- Cleanup advice never recommends copying and deleting (`the_exhausted_names_advice_does_not_recommend_archiving_and_names_the_open_file_risk`).
- Finish: a failed durability barrier stops the command and a refusal survives it; only a journal that cannot be removed after its
  durable marker is a warning (`an_accepted_run_whose_outputs_cannot_be_forced_to_disk_stops_the_command`,
  `a_refusal_with_a_failed_durability_barrier_keeps_the_refusal_and_names_the_barrier`,
  `an_accepted_run_whose_journal_cannot_be_removed_afterwards_still_passes`).
- Journals are listed after the workspace lock; an unreadable or malformed group record is a survivor; a survivor of one journal
  holds back every recovery and every new transaction (`a_group_record_that_cannot_be_read_defers_the_recovery`,
  `a_malformed_group_record_defers_the_recovery`, `a_survivor_of_one_journal_stops_the_recovery_of_another`,
  `a_new_transaction_does_not_start_beside_a_survivor`).
  Not tested: a journal published between the listing and the lock (the order is by construction now).
- A journal without a ready record is removed only when lockfiles, manifest snapshots and unknown files are all accounted for and the
  live files are durable; the ready record names exactly the payload the indexes imply
  (`an_unready_journal_whose_manifest_differs_from_its_snapshot_is_kept`, `an_unready_journal_that_lost_its_index_but_holds_snapshots_is_kept`,
  `an_unready_journal_is_kept_when_the_live_files_cannot_be_made_durable`, `a_ready_record_that_omits_a_manifest_snapshot_is_rejected`,
  `a_ready_record_with_a_duplicate_entry_is_rejected`, `a_ready_record_with_an_unexpected_entry_is_rejected`).
- The kept file's directory entries are synced up to the directory that holds the replaced file, which is the durable anchor
  (`every_ancestor_of_the_kept_directory_is_synced_up_to_the_directory_that_holds_the_target`).
- A failed lockfile root keeps the warnings of the others and does not stop their restore
  (`a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others`).
- Containment: the process tree, not the group, bounds a run (this process is a child subreaper while it runs);
  the child records its own group before it executes anything, and a child whose group cannot be recorded is not started;
  the record belongs to a scope owned by the transaction (nesting and any drop order are correct, and a transaction stays on its thread);
  pipe workers are cancelled when the call ends; a descendant that cannot be stopped makes install and update stop with the journal kept
  (`a_writer_in_another_session_is_stopped_before_a_normal_exit_returns`, `a_writer_in_another_session_is_stopped_at_the_bound`,
  `a_child_whose_group_cannot_be_recorded_is_not_started`, `nested_transactions_keep_their_own_records`,
  `a_descendant_in_another_session_that_holds_the_pipes_is_stopped_and_the_output_is_complete`,
  `a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal`).
  Zombies are not writers: the scan counts them as gone and reaps the adopted ones.
- Git reader: a fatal or signalled Git is an operational failure, not "not a commit"; system-configuration answers are reused only for a
  compiled program, keyed by its file identity and the variables that move the configuration, and the Git on the search path is the
  one `execvp` would run; a declined native attempt counts its starts; object files are opened without waiting and inspected on the
  descriptor; a commit is confirmed only under its own name (SHA-1), with Git's exact length rules and no bytes after a loose stream.
- The command named in errors is POSIX-quoted (`an_argument_with_a_space_is_quoted`).

Open picks, for the owner's veto:

- Containment by the process tree and `PR_SET_CHILD_SUBREAPER` instead of a control group; the cost is that a descendant that outlives a
  killed wrapper is found only through its recorded group.
- One bounded run in flight per process.
- The runner constants and the name-search ceilings are delegated picks, cited beside each value, not measurements.
- The "else background" half of "Force at leaving, else background" is still not built.
- A scan of `/proc` per run: 7.9 ms to 14.6 ms for 894 processes on 2026-11-04 (node, load average 6.7), twice per run at least.
