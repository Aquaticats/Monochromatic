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
  pick under the `Delegated picks of 2026-11-03` heading.
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
  The set-aside goes through the same transaction and ownership rules as the review findings under the
  Review findings fixed (cross-vendor review of 2026-11-03T13:26:08Z) heading,
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
  The catalog write and the `file-enforcer` sync are inside the transaction by a delegated pick
  under the `Delegated picks of 2026-11-03` heading.
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

The second review covered `504a82aa3f..09bb7820ea` in three parts (Git reader,
journal and restore,
install and policy).
The nine delegated picks under the `Delegated picks of 2026-11-03` heading stand;
the review found defects in how they were built,
not in the picks.
What changed:

- Ownership of an output file is decided in one place (`file_owner.rs`):
  the kernel's inode change time against the window of the cargo call,
  never by reading the file after the child returned.
  A file that differs from its snapshot and changed inside the window is the run's output;
  one changed later is another writer's and is never restored over.
  The remaining blind spot (a writer that publishes within about one tick of the call,
  or during it) is documented;
  the restore keeps every file it displaces,
  so it loses no data.
- The replacement protocol is a preservation contract (`journal_replace.rs`),
  not an atomic conditional replace:
  exclusive,
  synced temporary files;
  `RENAME_EXCHANGE`;
  the displaced file is read through its descriptor and hard-linked to a surviving name before anything is unlinked;
  an unexpected file is exchanged back and an edit made after the exchange is kept;
  a filesystem without exchange fails visibly;
  directories are synced.
- The journal is durable and claimable:
  snapshots are synced and the index is the commit record;
  recovery claims a journal by atomic rename and decides per root;
  the owner is a pid with its process start time;
  the manifests of a killed `--breaking` run are journaled in a lossless,
  versioned record.
- A failed cargo call that wrote a lockfile is rolled back like a successful one;
  a rollback that cannot complete stops the command (`InstallError::Unsettled`) instead of letting `--locked` build it;
  an unsupported registry is a policy refusal (`InstallError::Refused`) in every path.
- A lookup failure after a release-age refusal keeps the refusal (`check_policy`);
  frozen verification reports the non-strict age warnings;
  a malformed historical timestamp establishes no publisher history;
  `cargo update --precise` judges the release it names even when it was already locked (`precise_target.rs`).
- The placeholder lockfile is taken by rename before it is read;
  a failing floor sync restores the catalog only while it holds the raised text and records the manifests it rewrote.
- Update options need their values,
  `-m<path>` is understood,
  and the command word is found by position.
- A CSV row has a cumulative byte budget besides the field and field-count bounds.
- The Git reader follows the lexer of Git's `config.c`,
  reads the first `merge` value,
  refuses any `extensions.*` variable,
  worktree-private references,
  invalid or unusual refspecs and objects it cannot see to be commits,
  and asks Git once for every candidate when it declines.
  It is tested against git itself (`git_differential_tests.rs`).
- The system configuration path of the Git that answers is assumed to be `/etc/gitconfig`;
  finding it needs a git start,
  so this is a stated limit and not fixed.
  Power-loss durability is not tested with fault injection.

## Review findings of the third cross-vendor review (started 2026-11-03T17:33:52Z)

The third review found the replace protocol and the journal still failing on the same schedules,
so the protocol changed instead of
getting another comparison.
Where an earlier section of this record says a finding was "fixed" and this section says otherwise,
this section is current.

- Ownership is no longer what protects data.
  A restore never unlinks an inode that was live at a path (the displaced file and the replacement each get a surviving name first),
  a file that vanished stays vanished,
  and `ctime` only chooses between "restore and keep the displaced file" and "leave alone".
  Misattribution moves a foreign file aside with a warning,
  or leaves cargo's output live,
  which stops the command (`Unsettled`).
  This is the design the coordinator preferred on 2026-11-03:
  correctness that does not depend on attribution.
- Liveness is a lock,
  not a guess (delegated pick,
  open to the owner's veto).
  Every run holds one exclusive `flock` on a file in its own uniquely named journal;
  a recovery tries for it without waiting.
  The owner's rule "Use as few locks as possible."
  allows one per run;
  it replaces the process-id directory name,
  the `/proc` start
  time and the claim rename,
  which the review showed fail open (reuse,
  unreadable `/proc`,
  directory-name reuse,
  overlapping live runs).
  A recovery skips roots a live journal covers.
  A `ready` record,
  written last,
  lists every journal file with its length.
- Durability:
  file data before it is relied on,
  each new directory with its parent,
  kept outputs before the journal goes,
  a `.done`
  marker before removal.
  There is still no fault-injection test,
  so no power-loss guarantee is claimed.
- The owner's rule of 2026-11-03,
  "nothing should ever wait forever or indefinitely",
  is applied to every wait:
  locks are tried,
  child processes run under bounds that end in an error naming the call and the command to run again (cargo 30 minutes,
  Git 60
  seconds,
  `file-enforcer write` 5 minutes,
  a pause for HTTP 429 at most 2 minutes,
  the dump download an hour),
  with every file kept.
  These are chosen limits,
  not measurements,
  and they are delegated picks open to veto.
  The one exemption is the user's own command after the checks.
- An update verifies the lockfiles it leaves the way an install does (every locked version,
  or every version the base did not hold
  under `trustLockfile`);
  a rollback that cannot complete stops the command even when the policy report is the reason;
  the stand-in lockfile is kept before its name is removed;
  an owners lookup failure leaves a provisional finding unreported;
  a local pass with a warning is not remembered;
  a recording error around the sync still puts the catalog back.
- The Git reader sees symbolic-link references,
  the remote `.` mapping,
  exact path text,
  whole objects and all remote configuration,
  declines for repositories another user owns and for system configurations in other known places,
  and every Git call is bounded
  and reaped.
  Not fixed:
  a system configuration in a place no list knows of.
- `cargo add` and `cargo remove` are an explicit exclusion:
  they run cargo first and only warn;
  the policy is enforced by the install
  step of the next command that resolves dependencies,
  and only if that command goes through the shim.
  Dispatch tests for the
  whole table of verbs are not written (round 6:
  open).

## Review findings of the fourth cross-vendor review (started 2026-11-03T18:56:11Z)

- One workspace lock replaces the per-journal lock (delegated pick,
  open to the owner's veto;
  "Use as few locks as possible.").
  A transaction and a recovery both take it without waiting,
  so a recovery cannot race a transaction that starts,
  and a
  transaction that finds it held stops with the holder's process id.
  The lock is close-on-exec,
  because an inherited descriptor kept by a daemon that cargo starts would make the workspace
  unrecoverable indefinitely;
  the children of a killed wrapper are found by the process groups recorded in the journal instead.
- Every child runs in its own process group;
  at a bound the group is killed and shown to be gone,
  a normal exit kills what is left in
  the group,
  and output is returned only when complete.
- Finishing a run forces its outputs to disk before the completion marker (the owner's durability answer "Force at leaving,
  else
  background");
  the background half (syncing what a command did not promise) is not built.
- Kept files are never deleted or refused over by the tool (owner answer of round 6,
  "Warn past a count,
  never refuse (Recommended)"):
  past 1,000 files every run prints the count,
  the total size and the folder;
  the 1,000 is a warning point open to veto.
- The system configuration of the Git reader is what the answering Git reports;
  the answer is reused only for a compiled program
  (round 6 replaced the wrapper-blind cache key).
- Limits are measured or labeled unmeasured beside their values (repository rule LQM);
  the numbers measured on 2026-11-04 are in
  the README.
- pnpm oracle (pnpm 12.16.1,
  `minimumReleaseAge: 5256000`,
  a lockfile pinning `is-odd@3.0.1`,
  whose dependency `is-number@6.0.0` is
  also younger than the cutoff):
  `pnpm install --lockfile-only` fails with `ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION`;
  `pnpm update`
  fails the same way (and,
  when the policy is not strict,
  first adds the entries to `minimumReleaseAgeExclude`);
  `pnpm update
  is-odd` fails with `ERR_PNPM_NO_MATURE_MATCHING_VERSION` when strict and with the same violation for `is-number@6.0.0` when not.
  So pnpm,
  too,
  refuses an unchanged full update that leaves a too-young version locked in this configuration.
  Round 6 narrows the claim:
  the oracle shows that for a full update and for a target whose own dependency is too young.
  It does not show what pnpm does for an unrelated refused package outside a targeted update's dependency closure (cargo-shadow
  verifies the whole final lockfile,
  which can be stricter),
  for ordinary,
  latest and precise selection,
  for a version trusted at the
  configured base,
  or across unrelated workspace roots.
  Those cases are open,
  and the difference is a candidate for the owner's veto.
  Another difference:
  pnpm's non-strict mode edits `minimumReleaseAgeExclude` as a side effect,
  and cargo-shadow does not.

## Delegated picks of 2026-11-03

The owner is away for about 8 hours and delegated the open questions of the review work:
"Pick options as if I had picked them and invite veto when I say I'm back."
These are the coordinator's picks under that delegation,
not owner answers,
and the owner's veto is open when they are back.
Owner answer of 2026-11-04 (question tool):
"Keep all nine (Recommended)".
All nine picks of this section are owner decisions now,
and none is veto-open any more.

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
- A strict cutoff with exemptions keeps the approach recorded in the
  Review findings fixed (second cross-vendor review,
  started 2026-11-03T16:25:50.931Z) section:
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

Owner answer,
round 6:
"Warn past a count,
never refuse (Recommended)".
Past 1,000 kept files every run prints the count,
the total size and the folder;
nothing is refused or deleted.
The 1,000 is a warning point carried over from round 5 and open to veto;
the rate at which files are kept was not measured.

Fixed,
each with a red test committed before the fix:

- Kept folder:
  a notice instead of a refusal (`a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses`),
  and an unreadable folder is reported instead of read as small.
  The notice counts the kept folder only;
  files kept beside their target and stranded temporaries are reported by the recovery
  (`report_leftover_temporaries`),
  so a reservation of capacity before a transaction is overtaken by the owner's answer.
- Cleanup advice never recommends copying and deleting (`the_exhausted_names_advice_does_not_recommend_archiving_and_names_the_open_file_risk`).
- Finish:
  a failed durability barrier stops the command and a refusal survives it;
  only a journal that cannot be removed after its
  durable marker is a warning (`an_accepted_run_whose_outputs_cannot_be_forced_to_disk_stops_the_command`,
  `a_refusal_with_a_failed_durability_barrier_keeps_the_refusal_and_names_the_barrier`,
  `an_accepted_run_whose_journal_cannot_be_removed_afterwards_still_passes`).
- Journals are listed after the workspace lock;
  an unreadable or malformed group record is a survivor;
  a survivor of one journal
  holds back every recovery and every new transaction (`a_group_record_that_cannot_be_read_defers_the_recovery`,
  `a_malformed_group_record_defers_the_recovery`,
  `a_survivor_of_one_journal_stops_the_recovery_of_another`,
  `a_new_transaction_does_not_start_beside_a_survivor`).
  Not tested:
  a journal published between the listing and the lock (the order is by construction now).
- A journal without a ready record is removed only when lockfiles,
  manifest snapshots and unknown files are all accounted for and the
  live files are durable;
  the ready record names exactly the payload the indexes imply
  (`an_unready_journal_whose_manifest_differs_from_its_snapshot_is_kept`,
  `an_unready_journal_that_lost_its_index_but_holds_snapshots_is_kept`,
  `an_unready_journal_is_kept_when_the_live_files_cannot_be_made_durable`,
  `a_ready_record_that_omits_a_manifest_snapshot_is_rejected`,
  `a_ready_record_with_a_duplicate_entry_is_rejected`,
  `a_ready_record_with_an_unexpected_entry_is_rejected`).
- The kept file's directory entries are synced up to the directory that holds the replaced file,
  which is the durable anchor
  (`every_ancestor_of_the_kept_directory_is_synced_up_to_the_directory_that_holds_the_target`).
- A failed lockfile root keeps the warnings of the others and does not stop their restore
  (`a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others`).
- Containment:
  the process tree,
  not the group,
  bounds a run (this process is a child subreaper while it runs);
  the child records its own group before it executes anything,
  and a child whose group cannot be recorded is not started;
  the record belongs to a scope owned by the transaction (nesting and any drop order are correct,
  and a transaction stays on its thread);
  pipe workers are cancelled when the call ends;
  a descendant that cannot be stopped makes install and update stop with the journal kept
  (`a_writer_in_another_session_is_stopped_before_a_normal_exit_returns`,
  `a_writer_in_another_session_is_stopped_at_the_bound`,
  `a_child_whose_group_cannot_be_recorded_is_not_started`,
  `nested_transactions_keep_their_own_records`,
  `a_descendant_in_another_session_that_holds_the_pipes_is_stopped_and_the_output_is_complete`,
  `a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal`).
  Zombies are not writers:
  the scan counts them as gone and reaps the adopted ones.
- Git reader:
  a fatal or signalled Git is an operational failure,
  not "not a commit";
  system-configuration answers are reused only for a
  compiled program,
  keyed by its file identity and the variables that move the configuration,
  and the Git on the search path is the
  one `execvp` would run;
  a declined native attempt counts its starts;
  object files are opened without waiting and inspected on the
  descriptor;
  a commit is confirmed only under its own name (SHA-1),
  with Git's exact length rules and no bytes after a loose stream.
- The command named in errors is POSIX-quoted (`an_argument_with_a_space_is_quoted`).

Open picks,
for the owner's veto:

- Containment by the process tree and `PR_SET_CHILD_SUBREAPER` instead of a control group (probed available in the session that built
  this and not built);
  the cost is that a descendant that outlives a killed wrapper is found only through its recorded group.
- One bounded run in flight per process;
  a second run in the same process is refused at once,
  never waited for.
- The runner constants and the name-search ceilings are delegated picks,
  cited beside each value,
  not measurements.
- The "else background" half of "Force at leaving,
  else background" is still not built.
- A scan of `/proc` per run:
  7.9 ms to 14.6 ms for 894 processes on 2026-11-04 (node,
  load average 6.7),
  twice per run at least.

Not done in round 6,
so a later review does not read them as addressed:

- Cancellable pipe I/O has no test that fails without it;
  the one test written for it passed before the change,
  because the cancel path
  only matters when containment fails,
  which a test cannot provoke.
- Zombie handling under a PID 1 that does not reap is by design (the scan counts zombies as gone and reaps the adopted ones) and untested.
- The hard-link fault matrix (`EMLINK`,
  `ENOSPC`,
  `EXDEV`),
  a power-loss model,
  concurrent publication of a kept directory,
  and the
  three-writer undo.
- The kept-folder notice counts the kept folder only;
  names kept beside their target and stranded temporaries are not counted.
- A journal published between the listing and the lock,
  a wrapper killed while cargo writes,
  locks on network file systems,
  old and
  new binaries side by side,
  and the `add_manifests` lifecycle.
- The "else background" half of the durability answer;
  a test of the refused version already present in the initial lockfile.
- An oracle run of pnpm for an unrelated refused package during a targeted update;
  network tests for trickling,
  stalled and
  oversized bodies;
  add and remove dispatch tests.
- Per-line concept annotations of the new code (the tool that wrote them is a Python script,
  which the repository's script rules
  forbid,
  and it was not ported).

## Round 7: control group probes (run 2026-11-04T03:40:00-05:00 to 2026-11-04T04:05:00-05:00)

The coordinator replaced the process group (round 5) and the subreaper with a `/proc` scan (round 6) with a delegated control
group (cgroup v2):
cargo and everything it starts run in a group cargo-shadow owns,
stop is `cgroup.kill`,
quiescence is
`cgroup.events` reporting `populated 0`,
and a survivor after a crash is a membership query of the group the journal recorded.
The `Probe 1` to `Probe 5` subsections ran on this host:
Linux 7.2.15-ogc1.1.fc45.x86_64,
systemd 262.1-1.fc45,
cgroup2 mounted at `/sys/fs/cgroup`
with `nsdelegate,memory_recursiveprot`,
uid 1000.
Every probe ran inside `systemd-run --user --scope --property=CPUQuota=100% --property=MemoryMax=512M --property=TasksMax=200
--property=IOWeight=10`.
The scripts are scratch `.ts` files in `${HOME}/temp/agent/cargo-shadow-r7/` and passed cgroup paths as arguments,
never
interpolated into shell text (a path holds `\x2d` escapes that a shell ate on the first run,
which made the first
probe read an empty group).
The same facts are asserted by tests of the package (`cgroup_probe_tests.rs`),
each named after the probe number.

### Probe 1: can an unprivileged process get a group it may write?

Two ways,
and when each applies:

- A child of the process's own group.
  `mkdir` of `<own group>/probe-<pid>` succeeded and the new directory is owned by uid 1000.
  It needs the own group (from `/proc/self/cgroup`,
  hierarchy `0::`) to be owned by the user.
  True here for everything the user manager starts:
  the interactive terminal shells (`app-odytty@....service`),
  the harness
  scope `claude-code-bash`,
  and any `systemd-run --user` scope.
  False for a login session:
  `session-3.scope` is `root:root` mode 755 and `mkdir` there fails with `EACCES`.
  A read of `/proc/self/cgroup` is namespace-relative,
  so inside a container the path is relative to the container's cgroup
  root,
  and the same `mkdir` rule decides.
- A transient scope through the user manager:
  `systemd-run --user --scope --quiet --property=Delegate=yes true` is accepted
  and the scope is a user-owned delegated subtree.
  Its start cost 7.0 ms median (6.2 ms minimum,
  14.3 ms maximum,
  15 runs) against 1.9 ms for a plain `true`,
  so about 5 ms
  is added;
  it needs a running user manager (`XDG_RUNTIME_DIR`,
  a session bus),
  which a login session usually has and a
  plain container or CI runner does not.
  Not built in round 7:
  it needs the scope's path before the command starts and a gate between the two,
  and no P1 finding
  needs it.
  It is the next step for login sessions and is listed under "queued for round 8".

Result:
the writable child of the own group is built;
where `mkdir` or the later checks fail (not cgroup2,
not writable,
no `cgroup.kill`),
cargo-shadow takes the weaker path with a one-line notice (below).

**systemd's own rule makes the built way unsupported where the own group is not delegated.**
<https://systemd.io/CGROUP_DELEGATION/> (fetched 2026-11-04) says:
"Never create your own cgroups below arbitrary cgroups systemd
manages,
i.e cgroups you haven't set `Delegate=` in",
and "each cgroup only has a single writer,
i.e. a single process managing it".
The group a shell,
a terminal or a harness scope runs in is such a cgroup unless its unit has `Delegate=`,
so the child group
this tool makes there works on this host (every one of Probe 1 to Probe 5) but is not what systemd supports;
what systemd does with an
unknown child group of a non-delegated unit (a trim when the unit stops,
a change of controllers) was not probed.
The supported way is the second one:
a transient scope with `Delegate=yes`,
7.0 ms median to start,
which is where the
coordinator may want the default to move;
it is not built in round 7 because it needs the scope's path before the command starts
(`--unit=<name>` gives a predictable one) and a gate between the scope and the command,
and it needs the user manager.

### Probe 2: does `cgroup.events` report `populated 0` once every member is gone?

- After `cgroup.kill` on a group with an ordinary member,
  `populated 0` was reported within 5 ms and `cgroup.procs` was empty.
- **A leader thread that ended with `pthread_exit` while a second thread lives (the 6a R1 A counterexample) stays a member:**
  `/proc/<pid>/stat` shows `Z` for the leader,
  `cgroup.procs` lists the leader's id,
  `cgroup.threads` lists the live thread,
  and `cgroup.events` reports `populated 1`,
  so `populated` is correct and a `/proc` state test was not.
- **`cgroup.kill` alone did not end that group:**
  after 2000 ms (and in an earlier run 5000 ms) `populated 1` remained.
  `kill(2)` with `SIGKILL` on the id listed in `cgroup.procs` ended it in 1 ms,
  and so did `cgroup.kill` followed by `kill(2)`.
  The cause was not read in the kernel source;
  the guess is that `cgroup.kill` signals the listed leader task,
  which has
  exited,
  instead of the process,
  and only the measurement that `cgroup.kill` alone left `populated 1` is claimed.
  So the stop is `cgroup.kill`,
  then `SIGKILL` to every process listed by `cgroup.procs` (through a pidfd that is checked
  against the listing again),
  repeated until `populated 0` or the bound;
  this is built and tested.
- A zombie that nobody reaped,
  alone in a group,
  is not listed and `populated` is 0 (the kernel leaves the group when
  the task exits),
  so reaping the direct child stays its own `waitpid` with its own bound.
- Removing the group (`rmdir`) while a member lives fails with `EBUSY`.

### Probe 3: is a `setsid` plus double-fork escapee still a member?

Yes.
A member that ran `setsid sh -c '(sleep 600 &); sleep 600'` and whose grandchild was reparented to the user manager
(`ppid` 5698,
a different session) was still listed in `cgroup.procs`;
four processes in two sessions,
one group.
The limit,
which the review attached ("provided descendants cannot move themselves outside it"):
a member of the same uid
that writes its id into the parent group's `cgroup.procs` leaves the group (probe:
members left 0,
`populated 0`).
So the group contains accidental and ordinary escapes (`setsid`,
`setpgid`,
double forks,
closed pipes,
daemons),
not a
build script that deliberately moves itself out.
The probe has no reparenting handshake:
it read `cgroup.procs` without first observing the grandchild's `ppid` change,
so the membership is measured and the reparenting is read from the recorded parent process
number only (sixth pass,
group 1 wording).
The known closer is a cgroup namespace with the `nsdelegate` boundary,
which needs a user namespace and changes how cargo
runs;
it is not built and is the coordinator's to decide.

### Probe 4: does the group outlive cargo-shadow, and can its name be recorded and queried?

Yes,
while the enclosing unit lives.
A creator process made a group,
moved a `sleep` into it,
recorded the group's path and its directory inode (the cgroup id)
in a file and exited without killing.
A fresh process resolved the path,
found the same inode,
read `populated 1` and the member in `cgroup.procs`,
failed
`rmdir` with `EBUSY`,
killed the group,
saw `populated 0`,
and removed it.
The condition:
if the enclosing unit itself ends (the first run had the creator's own scope as the enclosing unit) systemd
stops the unit and signals its whole tree,
group included,
so the group was gone with its members,
which is also a safe
answer.
A recorded group that is absent,
or whose directory inode differs,
has no member (`rmdir` needs an empty group and a
new group gets a new id);
a present group with `populated 1` is a survivor.

### Probe 5: does a child started inside the group inherit membership without a hook?

Inheritance:
yes,
children and grandchildren (also those that `setsid`) are members from their first instruction.
Placement of the first process without `pre_exec`:
`std::process::Command` and `rustix` 1.1.6 have no `clone3` with
`CLONE_INTO_CGROUP` (grep of the installed sources found nothing).
The spawning process can enter the group by writing `0` to the group's `cgroup.procs`,
spawn,
and write `0` to its own
group's `cgroup.procs` again:
the child was in the group,
the parent was back in its own group,
and the group held the child
and its background `sleep`.
Measured in a node process with many threads:
enter 7.7 ms (the first migration takes the kernel's per-process
migration lock),
spawn 3.4 ms,
leave 0.04 ms.
This retires the `pre_exec` record path (6a R4,
R10):
the group is created and recorded before anything starts,
so there is
no per-run hook on the caller's `Command`.
The cost is that,
for the span of one `spawn`,
other threads of the wrapper that start a process would land in the group;
every spawn site of the package is `output_within` (one run per process),
so none does.

### The weaker path (coordinator's pick, veto open)

Where the own group is not writable or there is no cgroup v2,
a run is contained by a process group only,
as in round 5.
cargo-shadow prints one line naming the missing capability (for example "cargo-shadow:
no writable control group (mkdir in
<group> failed:
Permission denied);
descendants that leave the process group cannot be stopped or found") and the journal
records which containment was in force,
so a recovery never reads "the recorded process group is gone" as quiescence of a
journal whose containment was a control group,
nor the reverse.
No quiescence is claimed on this path:
the run reports the group stopped,
not the tree.

## Queued for round 8 (the P2 and P3 findings and the missing-test lists)

Not worked on in round 7 unless a P1 fix needed it.
Files are under `/var/home/user/temp/agent/reviews/`.

- `cargo-shadow-sol-6a.txt`:
  R4 (blocking opens;
  the `pre_exec` half is retired by the control group,
  the ELF probe open and
  the FIFO program remain),
  R5 (worker cancellation on every iteration),
  R6 (incomplete stdin as success),
  R7 (exit 1 with
  output),
  R8 (ELF header and the system-config cache),
  R9 (execute-bit test),
  R10 (retired by the control group),
  R11 (limit
  evidence for the runner constants),
  R12 (the generic error names the journal),
  and the P2 and P3 missing-test lists.
- `cargo-shadow-sol-6b.txt`:
  finding 2 (warning inventory outside the kept folder),
  finding 3 (authorization of the attempt
  limit),
  the hard-link and post-exchange fault matrix,
  reporting through actual callers,
  concurrency and ownership tests,
  the
  warning boundary test that creates one file instead of the count.
- `cargo-shadow-sol-6b2.txt`:
  findings 3 (ambiguous `*.tmp` in an unready journal),
  4 (lossy path conversion in the unready
  barrier),
  6 (finish does not check surviving writers),
  7 (the advertised repair for an unreadable group record),
  the
  missing tests for scope ownership,
  unready cleanup,
  integrity and deployment.
- `cargo-shadow-sol-6c.txt`:
  R3 (two restore failures aggregated),
  R4 (pnpm oracle for an unrelated refused dependency),
  R5 (non-UTF-8 arguments in the rerun text),
  the resource-limit assessment,
  and the missing regression coverage (shim
  dispatch,
  capture,
  policy,
  HTTP,
  name allocation,
  registration of `rerun_tests.rs`).
- Added by round 7:
  a transient user scope as the second way to a writable group (probe 1),
  the cgroup namespace closer
  (probe 3).

## Round 7: the P1 set of the sixth review (2026-11-04)

Where an earlier section of this record says the process tree,
the subreaper,
the recorded process group,
"one bounded run in
flight in a process" or "the directory that holds the replaced file is the durable anchor",
this section is current.
Each P1 below was written first as the reviewer's own counterexample and failed on the round 6 code (the commit with the red
tests is `939dcb31d1`:
eleven failed,
and the churn test,
which needs a lucky schedule,
passed);
the green commit follows it.

- 6a R1 A (a zombie leader with a live writer thread):
  `a_zombie_leader_with_a_live_writer_thread_is_stopped_before_a_normal_exit_returns`
  (a C fixture compiled by `cc` in the test;
  the leader ends with `pthread_exit`);
  the group is populated until stopped
  (`probe_2_a_thread_group_with_an_exited_leader_stays_populated_until_it_is_stopped`).
- 6a R1 B (a parent map that is not one snapshot):
  the parent map is gone;
  `orphans_of_parents_that_exit_while_the_run_ends_are_all_stopped` is a regression test that passed on the old code in most
  schedules,
  so there is no deterministic red for it.
- 6a R2 and 6c R2 (a killed wrapper and an escaped writer):
  `a_killed_wrapper_with_an_escaped_writer_stops_the_recovery` (the test
  kills the wrapper helper and the direct child and keeps a `setsid` writer alive;
  a recovery must not restore the lockfile);
  a recorded group with a member,
  an unreadable record,
  a lost record,
  another namespace and a weaker-path group are all
  survivors (`survivor_note`).
- 6a R3 (an ownership heuristic):
  `a_grandchild_adopted_during_the_run_that_the_run_did_not_start_survives` and
  `a_subprocess_another_thread_starts_during_the_run_survives`;
  the group is the run's own,
  nothing else is signalled.
- 6b 1 (the directory-entry sync):
  `every_ancestor_of_the_kept_directory_is_synced_up_to_the_mount_root` and
  `a_kept_directory_behind_a_symbolic_link_is_synced_by_its_resolved_path`;
  the walk is over the resolved absolute path up to the mount root,
  so it cannot end short,
  and a path that cannot be resolved fails the preservation.
  The walk stops at the mount root of the kept directory's file system (the first parent on another device),
  because an entry above
  it belongs to another file system (on this host the scratch root is on its own file system,
  `/` is a read-only composefs overlay
  on another device,
  and both `/home/user` and `/var/home/user` are mode 750 and owned by the user);
  an ancestor the user cannot
  open for reading,
  such as a shared directory of mode 711,
  fails the preservation as "kept but not made durable" and is a
  limit of this design.
  Not tested:
  a relative kept directory (resolving it needs the process's working directory,
  which a test cannot change safely
  while others run) and a power-loss model.
- 6b2 1 (survivor discovery that fails open):
  `a_journal_listing_that_fails_stops_the_recovery` and
  `a_journal_listing_that_fails_stops_a_new_transaction` (a failpoint stands for an entry or metadata error;
  `list_journals`
  propagates both,
  `fs::metadata` replaces `Path::is_dir`).
- 6b2 2 (missing group and ready records):
  `a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor`;
  positive
  counterparts `an_empty_journal_directory_is_no_survivor`,
  `a_journal_with_an_empty_group_record_is_no_survivor` and
  `a_recovery_removes_an_empty_journal_directory`.
  The protocol that makes "no child started" distinguishable:
  the `groups` file is the first file of every journal,
  and a record
  is forced to disk before every child.
  The advertised repair for an unreadable record (6b2 7,
  a P2) now says to restore by hand and remove the journal directory;
  an acknowledgement procedure is queued.
- 6b2 5 (outputs durable is not "only cleanup failed"):
  `a_completion_marker_that_cannot_be_published_stops_the_command`,
  `the_cleanup_failure_happens_after_the_completion_marker_was_published`,
  `a_completion_failure_is_not_cleanup_and_leaves_the_journal_active`,
  `a_removal_failure_after_the_marker_is_cleanup_only`;
  `FinishError` has three phases.
- 6c R1 (an operational install error bypasses the finish classification):
  `an_operational_error_with_a_failed_durability_barrier_stops_the_command_and_keeps_both_diagnostics` and,
  for the update,
  `an_update_error_with_a_failed_mandatory_phase_reports_both` (`settle_finish`).

Coordinator's picks under the delegation,
written beside the code they govern (veto open):

- Where no delegated group can be had,
  a run goes on in a process group with a one-line notice naming the capability
  (`bounded_run.rs`,
  `weaker_notice`);
  a recovery of a weaker-path journal asks only the recorded process group.
  This is weaker than the review's "fail closed on a journal whose quiescence cannot be established" (6c R2):
  a descendant that left
  the process group of a killed wrapper's run is not found there.
  The alternative the owner can pick instead is to refuse every recovery of such a journal until a person acknowledges it,
  which
  needs the acknowledgement procedure of 6b2 7.
- "Force at leaving,
  else background" has no background carrier in a command that exits:
  cargo-shadow forces what must be durable
  at its exit barrier and syncs nothing else (`Transaction::finish_with`).
- Every shipped limit cites a measurement,
  or the delegation record with the owner's quote (this round rewrote the comments of the
  runner constants,
  the HTTP and registry limits,
  the name searches,
  the set-aside rounds,
  the guard hop count and the
  dump ceilings).

Picks of the agent,
also open to veto:

- A run does not wait for another run:
  the old refusal of a second run in one process is gone;
  only the span of one `spawn`
  (the wrapper is inside the run's group for it) is serialized,
  with the run's bound.
- The wait for the child ends at its exit through a pidfd instead of a 10 ms sleep.
  Measured in the debug build,
  39 runs of `true` after one warm-up,
  on 2026-11-04:
  6.8 ms median with a control group,
  6.4 ms with a process group;
  the sleep-polling run took 11.3 ms at best.
  `clone3` with `CLONE_INTO_CGROUP` would avoid entering the group,
  and is not reachable from `std` or `rustix` 1.1.6.
- The tests that need a control group fail loudly where none can be had;
  `CARGO_SHADOW_ALLOW_NO_CGROUP=1` turns that into a printed
  skip for a host that cannot delegate (some CI runners and containers).

What the new design does not do,
so a later review does not read it as done:

- A same-uid member can leave the group (probe 3);
  the cgroup namespace closer is not built.
- A wrapper killed in the span of one `spawn` dies inside the new group;
  the group then holds only what the spawn started,
  which a recovery finds as a survivor,
  or nothing.
  Not tested (the window is about a millisecond).
- Login sessions (root-owned group) take the weaker path;
  a transient user scope is the second way to a group and is queued.
- Journal retention of `groups` lines grows by one line per run;
  the lines are removed with the journal.
- The weaker path writes "pgroup starting" before the child starts and the process group after (an advisor finding of round 7:
  the
  first form recorded only after the start and left a window in which a killed wrapper's child had no record);
  a journal with the
  first line alone is a possible survivor (`a_started_child_whose_process_group_was_never_recorded_is_a_possible_survivor`).
- The tests that need the thread-leader fixture compile a small C file with `cc`.

## Round 8 (2026-11-04): the contract first, then the code

The contract is `doc/planning/cargo-shadow-guarantees.md`;
its acceptance list is the reviewer's finite checklist,
and its coverage map
says which test shows each row.
Where the sections of rounds 5 to 7 say otherwise,
the contract and this section are current:

- The wrapper no longer enters a group around a spawn (7a F1,
  F2,
  F3 gone by construction);
  the program is started through
  `systemd-run --user --scope --property=Delegate=yes`,
  and the child-of-the-own-group route is dropped (it needs `clone3` with
  `CLONE_INTO_CGROUP`).
- The weaker path no longer lets a recovery go ahead on the disappearance of the recorded process group:
  it waits for
  `cargo-shadow acknowledge <journal directory>` (7c F1,
  7a F4).
- Recovery compares the mount id and inode of the journal and of the directory that held the recorded scope with what it sees,
  and
  refuses,
  naming the journal,
  when they differ (7a F6).
- An entry-level `NotFound` while listing the journals stops a recovery and a new transaction (7b P1-2);
  any failure on the
  containment path after the program started is `NotQuiescent` (7a F3).
- The kept-directory walk stops at the first directory that existed before the transaction,
  and goes through one this process knows
  it left unsynchronized (assumption b).
- The kept-files warning counts the names kept beside the lockfiles and the stranded temporaries too (row K1;
  this was queued in
  round 7).
- The queue of round 7 stays queued except what a row of the list needed:
  the K1 inventory above and the real completion rename and
  sync tests.

## Round 9 (2026-11-04): the answers to the round 8 reviews

Reviews answered:
`cargo-shadow-sol-8a.txt` and `cargo-shadow-sol-8b.txt` (in `/var/home/user/temp/agent/reviews/`).
The contract (`doc/planning/cargo-shadow-guarantees.md`) keeps its sixteen acceptance rows;
a counterexample reopened rows but added none.
Where the sections from What `cargo update` does under the cargo shadow to
Round 8 (2026-11-04):
the contract first,
then the code say otherwise,
the contract and this section are current:

- A run that recorded its end appends `settled <scope path>` to the journal;
  an absent recorded scope is read as gone only for a
  settled run,
  and an unsettled one waits for `cargo-shadow acknowledge` (8a F1).
- The journal's view record holds the machine id and the boot id;
  the same machine with another boot discharges process liveness
  only,
  and everything else (enumeration,
  snapshots,
  preservation,
  the mandatory finish,
  a scope with a member) still applies.
  New assumption e:
  one machine at a time,
  no checkpoint and restore (8b).
  A coordinator pick,
  veto open.
- The discovery of the manager's slice,
  the route cache and the registry of unsynchronized directories never wait:
  the first reads and
  reaps under deadlines,
  the other two use `try_lock` and answer conservatively (8a F2,
  F3).
- An acknowledgement counts only as the whole versioned record,
  made durable by the reader itself,
  and its writer reports success
  only after the barrier and the publication succeeded (8a F4).
- A crate root or central folder that cannot be listed gives an "unknown" kept-files warning,
  never a refusal and never "small" (8a K1).
- Tests:
  `src/round_nine_tests.rs` and the changes to the tests named in the coverage map;
  the 8b test fixes (each orphan with its
  own process id,
  the thread oracle by thread id with a control,
  ESRCH as gone,
  the completion-sync count,
  a namespace test that
  isolates the cgroup view,
  an R2 message that names a surviving path,
  a rollback test that asserts both faults) and the seven rows that
  were "partly" are now shown.
- Result at tip `7699f5df6f` (2026-11-04T07:37-05:00):
  561 tests,
  561 passed,
  3 skipped (the `#[ignore]` helper entries),
  debug and
  release gate alike;
  clippy with warnings denied is clean.
  Logs are named in the coverage map's Counts paragraph.
- Still the owner's to veto (kept):
  delegated transient scope as the default containment;
  the weaker path's recovery waits for
  `cargo-shadow acknowledge`;
  assumptions a to e;
  a verified-absent scope as proof of no members;
  the same machine id with a
  different boot id clears liveness only.
- First publication of the crate to crates.io needs the owner's explicit approval;
  nothing was published.

## Round 10 (2026-11-04): the answers to the round 9 reviews

Reviews answered:
`cargo-shadow-sol-9a.txt` (production code and documents;
"does not meet the list for this part") and
`cargo-shadow-sol-9b.txt` (tests;
"meets it after named fixes"),
both in `/var/home/user/temp/agent/reviews/`.
The contract (`doc/planning/cargo-shadow-guarantees.md`) keeps its sixteen acceptance rows;
both reviews said that no row is added,
and none is.
Owner decisions that bind this round (kept,
quoted):
the delegated transient scope as the default containment with the weaker path
waiting for `cargo-shadow acknowledge`;
a verified-absent scope as proof of no members;
a reboot of the same machine discharging
liveness only;
assumptions a to e;
forcing only at exit;
"nothing should ever wait forever or indefinitely";
"never discard user
data".
Where the sections from What `cargo update` does under the cargo shadow to
Round 9 (2026-11-04):
the answers to the round 8 reviews say otherwise,
the contract and this section are current.

### 9a D1: an unsettled scope is not completed by a later look

Defect:
`assess` read an unsettled scope as a completed lifecycle when `open.exists()` was true at the end,
so a scope whose control
group the manager had made and not yet attached the launcher to (`populated 0`),
or a scope that appeared between the query and the
existence check,
let a recovery restore files while the payload could still start.
Fix (`src/journal_unready.rs`):
every scope without a `settled` line is uncertain whether it is absent,
empty or appears during
the check;
only a populated scope is `Live`,
and no look at the scope upgrades an unsettled run.
Red then green:
`an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended` (the scope is made by hand inside a delegated
holder scope,
`populated 0` asserted) and `a_scope_that_appears_between_the_query_and_the_judgement_is_not_read_as_a_run_that_ended`
(a test hook inside `assess` between the queries and the judgement);
red at `6a6513b482`,
green at `a1c6ea1ae8`.

The way out of the refusal,
checked against "nothing should ever wait forever or indefinitely":
the refusal names the journal and the command `cargo-shadow acknowledge <journal directory>`;
the person's assertion overrides it (a
scope with a member still blocks),
the command tries the workspace lock and returns at once,
and no part of the program waits for
it.
The cost is real:
a wrapper that is killed before it records the end of a run now leaves a journal that waits for that one command,
also when the run's program ended long ago.

Options for removing that cost,
ranked (the owner is asked if the choice is not clear;
none of them is built):

- B,
  the launcher's liveness as a crash-safe proof.
  After the launcher is spawned (it becomes the program,
  same process) the wrapper appends `launcher <pid> <start time> <pid
  namespace> <scope path>` without a sync (a lost line only means uncertainty,
  and a power loss is a reboot,
  which the boot rule
  covers).
  A recovery first checks that the process is gone (no such process,
  or another start time,
  in the same pid namespace),
  and only
  then reads the scope:
  a launcher that is gone cannot attach itself later,
  so an absent or empty scope is final;
  a launcher that is
  alive blocks without an acknowledgement.
  Pros:
  killed runs recover by themselves in all but the window between the spawn and the line;
  keeps A's safety.
  Cons:
  a new record kind,
  a start-time parser of `/proc/<pid>/stat`,
  a pid-namespace comparison,
  zombies and thread groups with a
  dead leader to treat as not gone,
  and a manager that attaches a dead pid after the check needs a measurement.
- A,
  refuse and acknowledge (built).
  Pros:
  small,
  safe against both schedules,
  the way out is the existing command.
  Cons:
  every killed strong-path wrapper needs a person.
- C,
  a parent-death signal on the launcher so a killed wrapper kills it.
  Pros:
  ends the launcher with the wrapper.
  Cons:
  the signal does not reach the program's descendants,
  it is asynchronous so a recovery cannot read it as a proof,
  and a killed
  wrapper would then kill cargo,
  which changes the product.
Ranking:
  B > A > C,
  because B keeps A's refusal and removes the person from all but a narrow window,
  at the price of a protocol the
reviewer must check;
  A is complete and the smallest,
  but sends every killed run to a person;
  C proves nothing a recovery can read.
Built:
  A,
  because the review asked for the refusal and the narrower reading is the one to do first;
  B is proposed (built in round 11 at the owner's pick,
  see "Round 11").

### 9a D2: strict boot identity

Defect:
`read_identity_text` and `current_identity` checked only that the ids were not empty,
so a recorded `boot not-a-boot-id` beside
the right machine id differed from the real boot id and read as a reboot,
clearing a live process group.
Fix (`src/control_group.rs`,
`src/journal_unready.rs`):
the boot id must be a lowercase `8-4-4-4-12` UUID and the machine id 32
lowercase hexadecimal digits (the formats systemd reads,
`ID128_FORMAT_UUID` in `src/libsystemd/sd-id128/sd-id128.c`),
neither all
zero;
the view record is `view` then exactly one `boot` and one `machine` line,
and a repeated field or an unknown line leaves the
record without an identity;
the comparison also requires the current identity to be well formed.
Red then green:
`a_malformed_recorded_identity_is_not_a_reboot` (eight malformed records beside a live group,
and a well-formed record
of another boot as the control that clears),
`a_malformed_current_identity_is_not_a_reboot`,
`the_current_identity_is_read_strictly_from_its_files`;
red at `1318ffd101`,
green at `5d3466e2b1`.
The round 9 tests that built a different boot or machine id by appending text now change one digit (`after_a_reboot`).

### 9a D3: the discovery's one deadline

Defect:
after the deadline the discovery killed its program and then reaped it under a fresh two-second budget.
Fix:
`reap_within(child, deadline)` spends what is left of the discovery's own deadline (one non-blocking attempt even when it has
passed),
returns whether the child was reaped,
and the failure text says "the killed program could not be reaped before the deadline"
when it was not;
`DISCOVERY_REAP` is removed.
Red then green:
`a_discovery_that_cannot_reap_its_child_returns_at_the_deadline_and_says_so` (the `ReapStuck` failpoint makes the
killed child look unreapable;
red took 2.4 s against a 0.4 s bound);
red at `147f558d13`,
green at `7645642350`.

### 9a section 4: what a boot id is

The reviewer:
"Either enforce that authority or state the additional premise explicitly."
What Linux offers,
tried on 2026-11-04 in a disposable user and mount namespace (`unshare --user --map-root-user --mount`;
the host's
`/proc` was not touched;
scripts and logs `/var/home/user/temp/agent/cargo-shadow-r10/exp-boot.sh`,
`exp-boot2.sh`,
`logs/exp-boot.log`,
`logs/exp-boot2.log`):

- An unprivileged user can put a file over `/proc/sys/kernel/random/boot_id` with a bind mount,
  and the next read shows the new text.
  `statfs` on the path then reports the file system of the replacement (btrfs for a file of this host),
  not procfs.
- A bind mount of another procfs file (`/proc/sys/kernel/random/uuid`) keeps the procfs magic and a valid UUID,
  but it changes on
  every read;
  `findmnt` shows a mount at the path,
  and `statx` reports `STATX_ATTR_MOUNT_ROOT` for it.
- `stat --format=%m` does not help (it derives the mount point from the path and reported `/proc` every time).
- `systemd-nspawn` replaces the boot id on purpose,
  with a random id per container start,
  bound read-only over the file
  (`src/nspawn/nspawn.c`,
  `setup_boot_id_file` and `setup_boot_id`,
  read on the default branch on 2026-11-04).
  So a container-local boot identity exists in practice.
- A read-only bind of the directory that holds the file (container runtimes mask paths this way;
  tried with
  `/proc/sys/kernel/random`) leaves the file served by procfs and not a mount root.
  A bind of `/proc/sys` itself was refused by `mount` with exit status 32 in this namespace,
  so that form was not tried.

Options,
ranked:

- 1,
  enforce the file's authority and state what remains (built).
  `boot_id_authority`:
  the file is served by procfs (`statfs`) and is not the root of a mount (`statx`,
  `STATX_ATTR_MOUNT_ROOT`;
  a
  kernel that does not report the attribute is an error);
  an identity that fails is no identity.
  Pros:
  closes the reviewer's schedule (a replaced file on the same running machine),
  costs two syscalls per identity read,
  fails
  toward the acknowledgement,
  and also covers the `systemd-nspawn` case.
  Cons:
  a container whose runtime replaces the boot id gets the acknowledgement after a restart where a reboot clearance would have
  been right;
  a user-space `/proc` emulation (gVisor,
  an interposer) that reports procfs is not detected,
  and that residual premise is
  written into assumption c.
- 2,
  state the premise only.
  Pros:
  no code.
  Cons:
  the reviewer's schedule (a replaced file) stays open,
  and the cure is two syscalls.
- 3,
  corroborate with uptime and the wall clock (boot instant equals now minus uptime,
  compared with the recorded one).
  Pros:
  would also catch an emulated `/proc` that lies about the boot id but not about time.
  Cons:
  a clock step moves the instant in both directions,
  the check needs a tolerance with no measurement behind it,
  and an
  emulation can fake uptime too;
  it is evidence about time,
  not about the boot id file.
- 4,
  require the initial mount and pid namespaces.
  Pros:
  nothing can be put over the file in the initial mount namespace by a user.
  Cons:
  it removes the clearance for every process in a container,
  a toolbox or a user namespace,
  and the shortcut was the owner's
  pick for those cases too.
Ranking:
  1 > 2 > 3 > 4,
  because 1 answers the schedule at the cost of two syscalls where 2 only writes it down;
  2 is honest but
leaves the schedule open;
  3 adds a weaker kind of evidence with a tolerance nobody measured;
  4 is sound and removes the feature from
the places it is used.
Built:
  1;
  assumption c now carries the residual premise (the kernel that serves the process is the one that ran the writers,
  and its
procfs is not an emulation),
  and assumption e refers to it.
Red then green:
  `a_boot_id_put_over_the_kernels_file_is_not_an_identity` (in a new user and mount namespace:
  a bind of a regular
file,
  of another procfs file and a tmpfs over the directory each give "no identity" and keep a live group blocking;
  a read-only bind
of the directory is read as the real boot id,
  and the baseline likewise);
  red with the check disabled (log `logs/d4-red.log`),
green at `5d3466e2b1`.

### Part 9b

Fixed in the tests (commits `045187ed59`,
`5912ec62b3`,
`390bb75103`,
`74d035ca0e`,
`f796663c42`,
and the V2 arm named below):

- 3.1:
  the replacement inode is captured at the exchange (`replace_capturing`) and must keep its name at the live path after each
  fault.
- 3.2:
  a reported preservation needs the `.displaced` copy beside the target and a successful barrier of that directory,
  and the
  branch must run for at least one failed barrier.
- 3.3:
  the new refused copy is read and must hold `output of {first}`.
- 3.4:
  the settled-run test waits until the recorded scope is absent before the recovery reads it.
- 3.5:
  the thread and orphan tests require the program's group to equal the scope the journal recorded and to differ from the
  wrapper's own;
  the two escaped-writer tests assert the scope record of their run.
- 3.6:
  the record-fault tests assert that the fault was consumed and that the refusal is the injected one;
  the acknowledgement
  reader's file sync and directory sync are failed on their own (two new failpoints),
  each reached;
  an interrupted publication is
  built by hand from a real acknowledgement.
- 3.7:
  `is_running` treats only `ESRCH` as death,
  the thread fixture publishes a complete thread id by rename,
  and the discovery
  fixture records the descendant and the direct program and checks both at the return.
- 2.4:
  `a_bind_mount_over_the_directory_of_a_recorded_scope_is_refused_in_the_same_cgroup_namespace` (the helper first gives the
  journal the views of its namespace and the verdict clears;
  then it binds the parent directory of the slice over the slice,
  and the
  verdict refuses for the parent view).
  The guard removed:
  the test failed (`logs/n24-red.log`).
- One test needed a longer bound:
  with a journal around it,
  the escaped-writer test at the bound spent its one second in the
  record's sync and the manager's start when the whole suite ran at once (release gate,
  first run),
  so its bound is eight seconds.

V2,
"every other check" (9b section 2):
`a_reboot_still_publishes_the_completion_marker_of_the_recovery_or_keeps_the_journal` fails
the completion rename after a reboot discharge and requires the journal kept,
the failure reported and a control that settles it.

Not done from 9b section 2,
kept in the coverage map as "partly":
the shim and a user-command sentinel behind the A2/R4 call-site
test;
cuts of a wrapper at the real boundaries of `begin` (V3/V4).
Not done either:
the writer's side of the acknowledgement has no independent fault for the file sync inside `atomic_write` (only
the publication and the directory sync are faulted);
the reader's file and directory syncs are separate.

### Result

The coverage map of the contract carries the counts:
571 tests,
571 passed,
0 failed,
5 skipped (the `#[ignore]` helper entries),
for
the debug run and the release gate;
`lint:clippy` with warnings denied is clean.
Logs in `/var/home/user/temp/agent/cargo-shadow-r10/logs/`:
`r10-debug3.log`,
`r10-gate3.log`,
`r10-clippy4.log`;
the first gate
run (`r10-gate1.log`) failed once on the bound named above and is kept as evidence.
Nothing was published and nothing was merged into main;
the first publication needs the owner's explicit approval.

## Round 11 (2026-11-04): the launcher record

The owner's answers of 2026-11-04 (question tool),
quoted:
for the way out of 9a D1,
"Record the launcher (Recommended)" (after
starting the launcher,
cargo-shadow notes its process id,
start time and namespace;
recovery first checks that process is gone;
if it
is,
it can never join the scope later,
so recovery continues by itself;
a launcher still alive keeps blocking);
for the boot
authority,
"Check and state premise (Recommended)",
which is what round 10 built and is kept unchanged.
Option B of "9a D1" is therefore built;
options A (refuse and acknowledge) stays as the fallback for what B cannot prove,
and C
is not built.

What was built:

- `launcher <pid> <start time> <pid namespace> <scope path>` is appended to the journal's `groups` file right after the launcher is
  spawned (`note_launcher` in `src/bounded_run.rs`).
  The process id and the start time come from `/proc/<pid>/stat` (field 22,
  counted after the last closing parenthesis,
  because the
  name field may hold spaces and brackets);
  the namespace is the inode of `/proc/self/ns/pid`.
- Durability,
  as the owner asked ("if no sync is enough,
  prove it"):
  no sync.
  The line is one `write`,
  so a killed wrapper leaves it whole or absent,
  and the page cache keeps what a process wrote;
  `a_killed_wrapper_recovers_by_itself_once_its_launcher_is_gone` kills a real wrapper and reads the line back.
  After a power loss the line may be lost or torn:
  a lost line means an unsettled scope without a launcher line,
  which waits for an
  acknowledgement like an older journal,
  and a torn line is not understood or names another namespace or path;
  `a_torn_launcher_line_never_reads_as_a_gone_launcher` cuts a valid line at every length and requires the journal to stay blocked
  each time,
  with the whole line as the control that clears it.
  The synced `settled` line that follows carries the earlier lines to disk with it,
  and a reboot is decided first by the boot rule.
  So a sync would buy nothing a recovery can use and cost one more `fsync` on the start path of every run.
- A recovery (`assess` in `src/journal_unready.rs`) reads the whole record first,
  then for each scope without a `settled` line asks
  `launcher_state` before it reads the scope:
  gone means this process is in the recorded process id namespace and no process with
  that id and start time exists;
  an unreaped zombie counts as existing;
  another namespace,
  a process that exists but cannot be
  inspected,
  or a state that is not understood is unknown.
  Alive:
  blocks as live,
  no acknowledgement overrides it.
  Gone:
  the scope's own answer is final (populated blocks;
  absent or empty is ended).
  Unknown,
  no launcher line,
  a line for a scope the journal does not list,
  or a line that is not understood:
  uncertain,
  which
  `cargo-shadow acknowledge` overrides.
- What B cannot prove,
  and so still waits for a person (listed in the contract,
  A1):
  another process id namespace,
  a journal with no
  launcher line (an older version,
  a failed append,
  a wrapper killed between the spawn and the append),
  a line for another scope,
  a
  torn or not understood line,
  a launcher that cannot be inspected.

Red then green (red at `37c5524be1`,
on a scaffold at `a94403ae25` that parsed the record and ignored it;
green at `e6fab10dd4`):

- A killed wrapper whose launcher ended long ago now recovers without acknowledgement:
  `a_killed_wrapper_recovers_by_itself_once_its_launcher_is_gone` (a real wrapper helper is killed while its program runs;
  the
  recovery blocks with "still running" while the program lives,
  and settles the journal and restores the lockfile once the program
  ended and the manager collected the scope),
  and `a_gone_launcher_lets_the_recovery_read_the_scope_as_final` (planted journal:
  absent and empty scopes).
- A launcher still alive is not read as gone:
  `a_live_launcher_blocks_whether_its_scope_is_absent_or_empty` (absent,
  empty,
  and
  after an acknowledgement;
  the two round 10 D1 schedules stay refused while the launcher lives).
  A reused process id (another start time) reads as gone in `a_gone_launcher_lets_the_recovery_read_the_scope_as_final`,
  and in
  `the_launcher_question_tells_gone_from_alive_reused_other_namespace_and_zombie`;
  another process id namespace is unknown there and
  in `what_the_launcher_record_cannot_prove_waits_for_an_acknowledgement`;
  an unreaped zombie is alive.
- The round 10 D1 schedules without a launcher line stay refused (`an_unsettled_scope_that_exists_empty_...`,
  `a_scope_that_appears_between_the_query_and_the_judgement_...`,
  unchanged and green).
- Two older tests counted the lines of a run's record (scope,
  then settled) and now expect scope,
  launcher,
  settled.

Not measured:
the premise that a launcher which is gone cannot join the scope is the process model (an ended process has no thread to
attach);
no test makes the manager handle a request after its sender died.
The planted-journal tests use a stand-in process for the launcher;
the end-to-end test uses the real `systemd-run` launcher.

### Result

The coverage map of the contract carries the counts:
578 tests,
578 passed,
0 failed,
6 skipped (the `#[ignore]` helper entries),
for the debug run and the release gate;
`lint:clippy` with warnings denied is clean.
Logs in `/var/home/user/temp/agent/cargo-shadow-r10/logs/`:
`r11-debug1.log`,
`r11-gate1.log`,
`r11-clippy3.log`.
Nothing was published and nothing was merged into main.

## Round 12 (2026-11-04): the answers to the round 10 and 11 reviews

Reviews answered:
`cargo-shadow-sol-11a.txt` (production,
"does not meet the list for this part") and `cargo-shadow-sol-11b.txt` (tests,
"meets it after named fixes"),
in `/var/home/user/temp/agent/reviews/`;
both say no acceptance row is needed,
and none is added.

### 11a

- The launcher check and the procfs of another namespace.
  `kill` and `/proc/<pid>/stat` read a process id in different namespaces when a child namespace keeps an ancestor's procfs,
  and
  `/proc/self/ns/pid` names only the caller's,
  so the check could read a live launcher as a reused id and say "gone".
  Fix (`start_of` in `src/control_group.rs`):
  `pidfd_open(pid)` names the process in the caller's namespace,
  and the `Pid:` line of
  `/proc/self/fdinfo/<fd>` (read through the procfs) gives the number this procfs has for that very process;
  only when it equals the
  id is `/proc/<pid>/stat` read,
  otherwise the answer is unknown.
  The same function reads the start time when the line is written.
  No process with that id in the caller's namespace is gone without any procfs.
  If the entry has vanished after the match,
  a second look at the pidfd's number (it reads -1 for an ended process) separates "ended"
  from "hidden".
  Red then green:
  `a_procfs_of_another_pid_namespace_never_reads_as_a_gone_launcher` (a helper as the first process of a new user and
  pid namespace;
  with the ancestor's procfs the start time is not recorded and the question is unknown,
  with a procfs of its own the
  control moves:
  alive,
  a reused id and an ended launcher are gone).
  Red `301faaf776`,
  green `87da96a05f`.
- The boot id authority followed a symbolic link and raced with the read.
  Fix:
  `read_authoritative_boot_id` opens the file with `O_NOFOLLOW`,
  runs `fstatfs` and `statx` (`AT_EMPTY_PATH`) on that descriptor,
  and reads from it.
  Red then green:
  the `symlink` mode of `a_boot_id_put_over_the_kernels_file_is_not_an_identity` (the genuine directory is saved at
  another path and a replacement directory whose `boot_id` is a link to the genuine `uuid` is bound over the real one).
  The race has no test:
  no test makes a mount appear between the checks and the read;
  it is closed by construction.
- `reap_within` counted every `try_wait` error as a reap.
  Fix:
  only `Ok(Some(_))` is a reap;
  a failed wait keeps trying until the deadline and then reports the program as not reaped.
  Red then green:
  `a_wait_that_fails_is_not_a_reap` (a new `ReapError` failpoint).
- `read_identity_text` accepted `view nonsense`.
  Fix:
  the first line must be exactly `view <mount>:<inode>` with decimal numbers.
  Red then green:
  `a_view_header_that_is_not_mount_and_inode_gives_no_identity` (ten malformed headers,
  and a whole header as the control).

### 11b

- 2.1,
  one deadline against two budgets:
  the bound is 1.5 s,
  the return must come within 0.7 s of it.
- 2.2,
  K2:
  one ordered log of links and barriers;
  the protected name must be published by a logged link and a successful barrier of the
  target directory must follow it;
  both inodes are inventoried after every failure.
- 2.3,
  the real launcher:
  the payload publishes its own process id,
  waits at a release gate,
  and the record's process id,
  start time,
  namespace and scope are compared with it before the wrapper is killed;
  the kill waits for a whole launcher line;
  the launcher's
  disappearance and the scope's collection are awaited separately;
  the journal must be removed.
- 2.4,
  `a_gone_launcher_does_not_clear_a_scope_with_a_member` (with an acknowledgement that does not override),
  and the live launcher
  with an empty scope is acknowledged too.
- 2.5,
  the reboot control must end with no journal.
- 2.6,
  `a_torn_launcher_line_never_reads_as_a_gone_launcher` cuts the bytes production appends at every length,
  including the text
  without its line feed.
  That cut was red:
  a launcher line complete but for its line feed was read as a whole line.
  The production rule now is that a last launcher line without its line feed is not understood (commit `8ea7a0b63c`,
  red log
  `logs/r12-red2.log`).
  The writer's file sync is faulted (`an_acknowledgement_whose_file_cannot_be_synced_is_not_published`,
  a new `PublicationFileSync`
  failpoint in `create_exclusive`),
  and a real publisher is killed between its finished temporary file and the rename
  (`a_publisher_killed_before_its_rename_leaves_the_journal_unacknowledged`,
  a test hook in `atomic_write`).
- 2.7,
  the discovery fixture:
  an observer thread establishes,
  before the deadline,
  that the descendant is ready and alive when the
  direct program has exited.

Not done:
nothing named in 11a or 11b is left open.
The rows A2/R4 (the shim and a sentinel) and V3/V4 (real preparation cuts) stay partly,
as before.

### Result

The coverage map of the contract carries the counts:
584 tests,
584 passed,
0 failed,
8 skipped (the `#[ignore]` helper entries),
for the debug run and the release gate;
`lint:clippy` with warnings denied is clean.
Logs in `/var/home/user/temp/agent/cargo-shadow-r10/logs/`:
`r12-debug1.log`,
`r12-gate1.log`,
`r12-clippy3.log`.
Nothing was published and nothing was merged into main.

## Round 13 (2026-11-04): the answers to the round 12 reviews

Reviews answered:
`cargo-shadow-sol-12a.txt` (production,
"does not meet the list for this part") and `cargo-shadow-sol-12b.txt` (tests,
"meets it after named fixes"),
in `/var/home/user/temp/agent/reviews/`;
neither adds a row,
and none is added.
The owner's decision,
quoted:
"Out of scope,
keep cheap guards (Recommended)":
"Extend assumption a:
no program running as you
deliberately changes mounts or `/proc` while recovery runs.
Guards that cost little (the pinned `/proc` read,
no-follow opens) stay;
the review then judges against that premise."

### 12a

- Assumption a of the contract is amended:
  no program running as the user deliberately changes mounts or `/proc` (bind mounts,
  overmounts,
  replacing a procfs view) while a run or a recovery is in progress,
  alongside the premise that run writers do not
  deliberately leave the containment.
- Classified as excluded by that assumption,
  not as defects:
  the schedule of replacing `/proc` between the pidfd's fdinfo check and the
  stat read (at recovery and at recording),
  and the overmount of `/proc/<pid>` with a native procfs directory of another process.
  Neither has a test,
  and the contract says so.
- Kept as cheap guards and built:
  `open_proc_root` opens `/proc` once,
  requires a procfs (`fstatfs`),
  a mount root (`statx`) and a
  `self` link that names the caller's own pid;
  `read_in_proc` reads every later file (`<own>/fdinfo/<pidfd>`,
  `<pid>/stat`) with
  `openat2` and `RESOLVE_BENEATH | RESOLVE_NO_XDEV | RESOLVE_NO_SYMLINKS` through that descriptor.
  Cost:
  a handful of system calls per question,
  asked only when a journal names an unsettled scope and once per launched program.
  Kernel floor:
  `openat2` is Linux 5.6;
  below it the lookup errors and the answer is unknown (never gone);
  the strong path needs 5.14
  anyway.
- Fixed because it needs no tampering:
  after a failed stat lookup,
  a changed positive number is unknown.
  `missing_entry_verdict` returns gone only for `-1`,
  the number the kernel prints for an ended,
  reaped process;
  measured by
  `the_fdinfo_of_a_pidfd_says_minus_one_only_for_a_reaped_process` (an unreaped zombie keeps its number).
  Red `69be31fb74`,
  green `77e48d7c5e`:
  `only_the_ended_process_number_shows_a_gone_process`.
- The contract,
  this record and the README no longer claim that a start time can never come from another process or that no procfs
  of another namespace can ever give "gone":
  they say what the guards cover and what assumption a excludes.

### 12b

- 3.1:
  the payload of the real-launcher test publishes its own `/proc/self/cgroup`;
  the test compares it with the scope the journal
  recorded (`assert_ran_in_recorded_scope`) and with the wrapper's own group,
  so the comparison is no longer the journal with itself.
- 3.2:
  the inherited-procfs case now records the readable ancestor entry under the launcher's number (its name must not be `sleep`)
  and requires the specific refusal ("another process id namespace"),
  not any inspection error.
- 3.3:
  after the second acknowledgement of the interrupted publication the journal must be gone,
  not only the lockfile restored.
- The two admitted gaps are closed:
  `a_wrapper_killed_after_each_step_of_the_preparation_is_recovered_conservatively` (V3/V4:
  a `cfg(test)` hook after each step of
  `begin`;
  a real wrapper is killed after the directory,
  the record,
  the view,
  the owner file,
  the snapshot,
  the index and the ready
  record;
  before any snapshot the journal is removed and the lockfile untouched,
  with a snapshot or an index and no ready record the
  journal is kept and the lockfile untouched,
  with the ready record the recovery restores and completes),
  and
  `a_run_that_cannot_show_quiescence_stops_the_shim_before_the_users_command` (A2/R4:
  `run_shim` for `cargo check` in a helper
  process,
  a stale lock,
  a fake cargo whose user command creates a sentinel;
  the second run of the install step,
  the update,
  cannot
  be stopped;
  the shim exits with the install-failed code,
  the sentinel is absent,
  the lock the update wrote is neither rolled back
  nor accepted,
  the journal is kept).
  Both passed at their first run of the final form,
  so they are evidence for rows that had hand-built or lower-level tests,
  not red
  controls.

### Result

The coverage map of the contract carries the counts:
588 tests,
588 passed,
0 failed,
10 skipped (the `#[ignore]` helper
entries),
for the debug run and the release gate;
`lint:clippy` with warnings denied is clean;
16 rows,
16 shown by the map's own
criterion,
none certified.
Logs in `/var/home/user/temp/agent/cargo-shadow-r10/logs/`:
`r13-debug1.log`,
`r13-gate1.log`,
`r13-clippy2.log`.
Nothing was published and nothing was merged into main.

## Round 14 (2026-11-04): the answers to the round 13 review

Review answered:
`cargo-shadow-sol-13.txt` ("does not meet the list";
every 12a finding closed or excluded by the amended assumption a,
the 12b repairs closed;
no row needed) in `/var/home/user/temp/agent/reviews/`.

### Section 3: the start time and the time namespace

Superseded in round 15:
the fix described here (a time namespace number in the launcher line,
compared at recovery) is gone;
a namespace
number is recycled,
so it proved nothing (see "Round 15").
What stays from round 14 is the measurement,
the discriminator test (tightened
in round 15) and the finding that field 22 is relative to the reader's time namespace.

Defect:
field 22 of `/proc/<pid>/stat` is relative to the boot-time offset of the time namespace of the reader,
so a recovery in a
time namespace prepared beforehand read a live launcher's start time differently and took the difference for a reused id.
This needs no remount,
so the owner's exclusion of round 13 does not cover it.
Measured here (`unshare --user --map-root-user --time --boottime 7200`):
the same process read 16018424 outside and 16738423 inside,
a difference of 720000 ticks for two hours.
Fix of round 14 (replaced in round 15):
the launcher line carries the inode of the writer's time namespace (`launcher <pid> <start> <pid ns> <time ns> <path>`);
the
question compares it with the reader's and answers unknown on a difference,
when either side cannot be read,
and for a line of an
older version that has none (it still parses,
with no time namespace).
Red then green:
`a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone`:
a helper in a new user and time namespace
(same process id,
mount and cgroup namespaces) with a two-hour offset reads a different start time for the held launcher (the
prerequisite,
asserted),
and both the question and a recovery of a planted journal answer unknown,
never gone;
the same helper
without the shift is the control (alive,
the start time equal).
It was red with the time namespace recorded and ignored (`logs/r14-red.log`).
The test needs `unshare` with permission for a user and time namespace;
this host allows it.
The other tests of the line were updated for the new field.

Other time-relative values:
none.
The boot rule uses the boot id and machine id;
the code reads no uptime,
`btime` or `CLOCK_BOOTTIME`;
the realtime clock used for the
acknowledgement time is not namespaced.
Only field 22 depends on the reader's time namespace.

### Section 1: the shim test

`a_run_that_cannot_show_quiescence_stops_the_shim_before_the_users_command` now reads the helper's journal and requires a scope
record,
requires the absence of the weaker-path notice,
requires that the armed `ContainmentStopStuck` was consumed (the helper
writes whether it was),
and requires the refusal text of a run that could not be shown empty ("could not be stopped,
so it may still
write the lockfiles"),
besides the exit code,
the absent sentinel,
the kept lock and journal.
With the fault never armed the test fails (the user's command runs and replaces the helper),
shown once with the arming moved aside.

### Result

The coverage map of the contract carries the counts:
590 tests,
590 passed,
0 failed,
11 skipped (the `#[ignore]` helper
entries),
for the debug run and the release gate;
`lint:clippy` with warnings denied is clean;
16 rows,
16 shown by the map's own
criterion,
none certified.
Logs in `/var/home/user/temp/agent/cargo-shadow-r10/logs/`:
`r14-debug1.log`,
`r14-gate1.log`,
`r14-clippy2.log`.
Nothing was published and nothing was merged into main.

## Round 15 (2026-11-04): the answers to the round 14 review

Review answered:
`cargo-shadow-sol-14.txt` ("does not meet the list for the changes;
not ready for the certification pass") in
`/var/home/user/temp/agent/reviews/`.
Closed per the review:
the original distinct-time-namespace schedule,
and the shim route and fault evidence.

### Section 2: the recycled time namespace number

Defect:
namespace inode numbers are eligible for recycling after the namespace's last reference is released,
and a writer's current time namespace can differ from
the one its launcher lives in,
so the number recorded in round 14 can name a later,
different namespace;
a recovery there read a
different start time and answered gone for a live launcher.
Reproduced on this host,
deterministically (`logs/r15-red.log`):
`a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone`.
A writer helper runs in a time namespace T0 (offset 3600 s) and starts its launcher in T1 (offset 7200 s);
the processes in T0 are
killed so that T0 has none;
a helper joins the launcher's user namespace (`nsenter --preserve-credentials --user=...`,
so that no new
user namespace takes the freed number first) and creates T2,
which gets T0's number.
Asserted prerequisites:
the launcher is alive,
the number is reused,
the recovery's own start time differs from the recorded one.
With the round 14 rule the question answered `Gone` and the recovery cleared the journal.
Options considered:
- 1,
  a start time that differs from the record is unknown (built).
  Pros:
  no kernel version,
  nothing recorded that can be recycled,
  nothing to retain after the wrapper dies;
  an id that no process has
  stays gone,
  which is the ordinary case of a launcher that ended.
  Cons:
  a reused id whose start time differs from the record (the pid counter wrapped since the launcher ended) waits for the
  acknowledgement;
  one whose start time happens to equal it is classified alive and blocks.
- 2,
  the pidfs inode number of the launcher's pidfd as an identity independent of every namespace,
  with option 1 below it.
  Verified in `fs/pidfs.c` (read on the default branch and at v6.9,
  v6.12,
  v6.14 on 2026-11-04):
  from 6.9 to 6.13 the inode number
  comes from an `ida` and is reused;
  from 6.14 it comes from a 64-bit counter that is not reused,
  and on a 32-bit kernel only its
  lower 32 bits are the inode number.
  Pros:
  would keep the reused-id case automatic on 6.14 and later on 64-bit.
  Cons:
  a kernel version gate and a second path to test,
  for a case that is rare.
- 3,
  a reference process (the start time of the namespace's first process,
  read by the writer and by the recovery) to prove a common
  basis.
  Pros:
  no kernel version.
  Cons:
  the offset is rounded to clock ticks,
  so equality of the reference does not prove equality of the launcher's rounding;
  it
  needs a tolerance and more tests.
Ranking:
  1 > 2 > 3,
  because 1 is sound on every kernel with the least code and costs only a rare acknowledgement;
  2 recovers that rare
case on new kernels at the price of a gate;
  3 adds a tolerance that has to be argued.
Built: 1.
What was removed:
  the time namespace number from the record (the format is the round 11 form again;
  a line of round 14's form is
not understood,
  so it waits for an acknowledgement),
  `time_namespace_id`,
  and the time namespace argument of the question.
`a_reused_...` tests of rounds 11 and 12 that asserted gone for another start time now assert unknown.

### Section 3

- The shifted reader's start time in `a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone` must be read
  successfully as a number and differ numerically from the recorded one (an `err` text no longer satisfies it).
- The recovery's diagnostic must name the start times (the refusal is the start-time comparison,
  with the other views equal).
  This held before the fix and holds after.

### Section 4: claims corrected

The contract,
the map and the update record no longer say that a time namespace number is compared;
round 14's section is marked
superseded;
the contract states why the number proves nothing and what replaced it.

### Result

The coverage map of the contract carries the counts:
592 tests,
592 passed,
0 failed,
12 skipped (the `#[ignore]` helper entries),
for the debug run and the release gate;
`lint:clippy` with warnings denied is clean;
16 rows,
16 shown by the map's own criterion,
none certified.
Logs in `/var/home/user/temp/agent/cargo-shadow-r10/logs/`:
`r15-debug2.log`,
`r15-gate2.log`,
`r15-clippy3.log`.
Nothing was published and nothing was merged into main.

## Round 16 (2026-11-04T11:17:02-05:00): the certification pass

Inputs answered:
`cargo-shadow-cert-sol-1.txt` (A1,
V1,
V2 amended c,
V6),
`cargo-shadow-cert-sol-2.txt` (A1 A2,
A1 V3,
A2 R4,
V3 V4) and
`cargo-shadow-cert-sol-3.txt` (A3 A4,
R1,
R2,
R3 R5,
K1,
K2,
V5,
weaker path),
in `/var/home/user/temp/agent/reviews/`.
Verdicts:
not certified;
no production defect established;
test discrimination gaps,
and one test-support defect (the fail-open
`journal_dirs` listing helper of group 3 and the fail-open process oracle of group 1).
The map marked A1,
A1/V3,
A2/R4,
V2,
V3/V4 and V6 partly until the fixes landed,
then back to shown with the fixes listed per row.

Production seams added for the tests (`cfg(test)` only,
no change of behavior):
an ordered trace of the recording and launching in
`bounded_run.rs`,
a hook after the program started that may fail the startup completion,
a `ContainmentSurvivor` fault that makes the
stop run its real loop to its deadline,
a fail-closed `process_is_running`,
the recovery's output-barrier and cleanup faults,
and
`QUIESCE_BOUND` visible to the tests.

What each group asked,
and what was done (the contract's map lists the test names;
the planted regressions are in its red-first list):

- Group 1.
  A1:
  the direct child is observed reaped by a checked `waitpid` (`ECHILD`) after a normal return and at the bound;
  the zombie-leader
  fixture is driven through the timeout.
  V2:
  a malformed-only record;
  a member in a scope (with a resolvable lifecycle) for recovery and admission,
  each with a control;
  the
  crash test classifies the launcher gone before it recovers and names the member;
  the changed-view cases and the bind substitution
  invoke the real recovery.
  V6:
  a stalled scope startup and an unsuccessful shutdown with a surviving member,
  each bounded in elapsed time;
  the two lock
  contenders answer in a window while the holder is held;
  the bounded-run oracle fails closed through the pinned procfs and the
  descendants tests record a delegated scope;
  the timeout test asserts `TimedOut` and the payload's start.
- Group 2.
  A1/V3:
  two launchers held at the launch boundary with an unrelated spawn between them,
  all in the wrapper's group,
  with the planted
  move as a control.
  A2/R4:
  a failed startup completion with a writer ready;
  every fault has a control,
  is consumed and has its ready marker;
  the shim
  fixture tracks arming apart from consumption.
  V3/V4:
  the ordered trace of write,
  whole-record sync,
  sync,
  launch;
  the containment kind attempted at a record fault.
- Group 3.
  A3/A4:
  an operational error with each completion stage;
  the joined update error leaves the operational classification.
  R1:
  the kept in-place file holds the writer's bytes and is the captured inode.
  R2:
  each queued error fires and both inodes are inventoried.
  R3/R5:
  the rollback test asserts the write,
  the consumed fault and the specific refusal.
  K1:
  the combined size and the folder,
  repeated runs,
  the inventory.
  K2:
  links and barriers in one log on the successful chain.
  V5:
  an active recovery's own finish with each failure and the cleanup-only case;
  the unready and ready tests of
  `journal_recover_tests.rs` are named in the map and included in the rebuilt bundle.
  Weaker path:
  the kernel without `cgroup.kill`,
  a scope the manager refuses,
  recovery with one of two journals acknowledged,
  the
  contended lock,
  the shutdown with a running writer and a `TimedOut` result.
  Shared:
  `journal_dirs` fails on every listing error but the initial absence.

Not done,
and why:
the stalled-startup,
the weaker-path and the rollback tests have no planted regression that turns an old assertion
red (the old tests did not exist or had no cheap plant);
their evidence is the assertions and the controls.
A real unsuccessful shutdown (a member in an uninterruptible wait) is not built:
the surviving member is a fault that makes the real
loop run to its deadline.

### Result

The coverage map of the contract carries the counts:
609 tests,
609 passed,
0 failed,
13 skipped,
for the debug run and the release
gate;
`lint:clippy` with warnings denied is clean;
16 rows,
16 shown by the map's own criterion,
none certified.
Logs in `/var/home/user/temp/agent/cargo-shadow-r10/logs/`:
`r16-debug1.log`,
`r16-gate2.log`,
`r16-clippy2.log`.
The certification bundles are rebuilt against the tip of this round.
Nothing was published and nothing was merged into main.

## Resume point (round 16, rewritten 2026-11-04T11:17:02-05:00 after the gate)

Done:
rounds 10 to 16 (the Round 10 (2026-11-04):
the answers to the round 9 reviews section
to the Round 16 (2026-11-04T11:17:02-05:00):
the certification pass section).
Open for the owner:
the picks of rounds 8 to 13 stay veto-open as listed in their sections;
the first publication needs the owner's
explicit approval.

Next:

1.  The coordinator launches the certification pass again against the rebuilt bundles.
2.  Nothing is merged into main and nothing is published.

## Owner decisions of 2026-11-04

- A cargo-shadow that is started by cargo-shadow stops at once.
  Question put to the owner:
  when PATH holds a launcher named `cargo` that has no `.launch` record and leads back to cargo-shadow,
  should the bounded fallback (up to six starts) stay,
  or should the second cargo-shadow exit 2 before doing any work,
  as the Git
  wrapper does (`package/git-policy/cli/SPEC.md`,
  section "The wrapper never starts itself as Git")?
  Answer:
  option A,
  the second cargo-shadow exits 2 before doing any work,
  with the owner's note,
  verbatim:
  "A,
  but make the help message descriptive enough that even a monkey can understand and execute it."
  Built in `package/cargo-shadow/cli/src/arrival.rs` and `shim.rs`:
  the message says in one sentence what happened,
  shows the chain
  with full paths,
  the PATH it saw and every `cargo` on it,
  why it stops,
  and two numbered fixes with lines that work as printed
  (an `export PATH=` line that puts the real cargo's folder first,
  and a `printf` line that writes the `.launch` record),
  and how to
  check (`cargo --version`).
  Tests:
  `tests/shim.rs`,
  `an_unrecorded_launcher_that_leads_back_stops_the_second_start_at_once_and_its_message_fixes_the_problem_when_copied`
  runs both fixes as printed and shows the command then works,
  and checks that nothing of the second start ran;
  `an_unrecorded_launcher_that_leads_back_stops_the_second_start_of_add_with_status_2` covers a start as a child.
  The bounded fallback and the hop limit are gone.
- The nine delegated picks of 2026-11-03 are kept (see the Delegated picks of 2026-11-03 section).

## Resume point (round 21, written 2026-11-04T17:51:00-05:00, paused by the owner)

Owner instruction,
verbatim:
"Don't discard work,
but aside from layout + linting fixes,
pause everything else - but let work already
in flight run to their natural conclusions.
Layout + linting fixes are the biggest priority."
Nothing was discarded;
the branch is
`feat/cargo-shadow-check-to-start-race` (worktree `.claude/worktrees/cargo-shadow-r21`),
not merged,
nothing published or released.
Clippy (`mise run //package/cargo-shadow/cli:lint:clippy`) was clean at the last commit.
The full release gate was not run in this
round,
and no packets were rebuilt.

Done in round 21 (each with its test named in `cargo-shadow-guarantees.md`,
section "Round 21 additions",
or in the commit message):

- Check-to-start race for compiled real cargos:
  `real_binary::vet` and `Vetted::command` start through the checked descriptor at the two
  starts this process makes itself;
  the contained cargo of an install step keeps the path with a repeated check (`command_by_path`),
  because the scope launcher starts the program from its own process.
  Script and contained-start residual is recorded in the contract.
- Sixth-pass group 2 and group 4 named fixes.
- Production defects:
  a zero or impossible process id no longer counts as an acknowledgement (group 13);
  the printed Fix B record is
  built by the JSON serializer (group 14);
  a failed rollback keeps the failure the run already had (group 5);
  the kept-files warning
  is now announced by the fast path and by a frozen run (group 7).
- Evidence fixes:
  listing faults replace the results of the real operations and completed journals are examined (group 5);
  record
  sync and the journal's own barriers take injected failures through the real result handling (group 12);
  reap tests compare the
  program's id with the spawned direct child (group 1);
  slow-start retry times its judged attempt (group 9);
  gone launcher with an
  existing empty scope and the launcher-uncertainty and time-namespace cases run through the real recovery and admission (groups 3 and
  10);
  collision-named refused copies count toward the warning (group 7);
  tests that cannot build their schedule fail unless
  `CARGO_SHADOW_ALLOW_SKIPS` is set.

Not done (resume here when the owner lifts the pause),
in the order the sixth pass asked (production defects,
false greens,
coverage):

1.  Run the full release gate with the installed shadow first on PATH (`$HOME/temp/agent/gate-final.sh`,
    already pointed at the r21
    worktree),
    then rebuild the packets as cert7 under 250 KB each with the manifest,
    and fix group 6's packet,
    which had no test
    bodies assigned.
2.  Group 5:
    manifest-specific output-barrier failure (path-targeted);
    refused output plus `CompletionSync` failure;
    independent
    syscall observation of the completion-parent barrier.
3.  Group 8:
    preferred-directory link failure followed by fallback-stage and barrier faults (R2,
    sibling fallback and stranded path);
    recovery output barriers at the real file and directory syncs and their ordering;
    post-exchange restore failure with the manifest;
    manifest unchanged on a damaged-ready rejection;
    update-caller continuation on a mandatory recovery failure;
    counted swap
    callbacks in the exchange tests.
4.  Group 9:
    K2 retry after a failed own-directory barrier and after a failed parent-entry barrier in the same process.
5.  Groups 10 and 11:
    negative boot-identity and replaced-identity cases through recovery and admission;
    journal-view admission;
    replace the masked legacy cross-journal and malformed-record tests with discriminating bodies;
    live-scope admission must assert the
    journal and snapshots are retained after the first refusal;
    parent-view substitution through admission.
6.  Group 12:
    pending-launcher preparation kill;
    delegated-case skip wording.
7.  Group 13:
    weaker-route record write and sync failures with the weaker route forced (no payload starts,
    append then barrier then
    spawn);
    acknowledgement barriers at the real operations and the reader's failures through recovery and admission;
    the CLI
    acknowledgement protocol (lock,
    durability,
    quoting);
    positive acknowledgement controls for the broader uncertainty classes.
8.  Group 14:
    depth-aware append-only entry trace proving the second shadow is terminal and does not enter transaction preparation;
    tests at the production start sites with a handshake between selection and check;
    remove the stale bounded-fallback test reference.
9.  Group 1 wording only:
    `assert_scope_record` proves the record,
    not placement;
    the double-fork probe has no reparenting handshake.
10. The fifth-pass open list not covered by items 1 to 9:
    R2 stranded path,
    admission schedules for the remaining boot and unreadable records,
    the acknowledgement writer's ordering,
    the mandatory-finish refusal through a real shim run,
    the blocked-feeder handshake,
    the
    start-time line naming the namespace,
    the `missing_entry_verdict` and `start_of` consumer wiring,
    and the group 1 refusal when the
    emptiness proof fails.
11. Contract:
    add suite counts to the round 21 block once the gate has run;
    map the new tests.

Planted-red notes for the final report:
the K1 collision test,
the listing faults,
the record sync,
the journal barriers,
the
empty-scope recovery,
the launcher uncertainty cases and the slow-start judged duration were each turned red by a planted change and
restored.
The two time-namespace plants (ignoring a start-time note in recovery or admission) went red;
the recovery-side plant failed
in under a second,
so its exact assertion was not inspected.
No cheap plant exists for the spawned-child id check (a launcher that
forks instead of replacing itself is not built) and for the `assert_ran_in_recorded_scope` additions of group 2 and 4 beyond the
earlier control.

## Round 21 merge record (2026-11-07)

The owner's pause of 2026-11-04 was lifted for this merge.
The branch `feat/cargo-shadow-check-to-start-race` took main twice (`0a3aa354ca`,
with 0.1.5-alpha.1,
and the merge of the locks for `monochromatic-lint` 0.5.2-alpha.2),
and the release gate ran for the first time on the round (the round 21 Resume point section says it had not).

- The gate found one red test that the round itself caused:
  the new `spawned` trace event sits between the launch and the launcher line,
  so `the_record_is_written_made_durable_and_only_then_the_program_is_launched` failed.
  It now expects the event (`f9dcf825b1`).
  It does not ignore it:
  a filter that dropped `spawned` events was refused by the permission classifier as a loosened test,
  and the stricter form pins the event right after the launch.
- The cross-vendor review (`~/temp/agent/reviews/cargo-shadow-r21-sol-1.txt`,
  verdict "merge after named fixes") found one
  production defect that no test had reached:
  the exec site bound the vetted program as `_vetted` inside a match arm,
  which dropped the open file before `exec`,
  so a compiled real cargo (rustup's proxy is one) failed to start on the main path.
  Red tests first (`b5e14dff0d`,
  three integration tests,
  two red),
  then the fix (`0a91265df1`).
  Two more findings,
  each red first:
  an execute-only compiled program was refused (the PATH search accepts it),
  fixed by an `O_PATH` open for the checks;
  the second review then showed that such a file cannot be told from an execute-only script,
  which the kernel runs and which must be given its own path,
  so an unreadable file is started by path with the check repeated (red test first,
  `an_execute_only_script_real_cargo_is_given_its_own_path_by_the_kernel`);
  the largest process id was inclusive although `pid_max` is one greater than the maximum PID (`proc_sys_kernel(5)`).
  The contained-start test now requires the scope route and asserts the scope launch;
  a planted `vetted.command()` in `RealCargo::run` turned it red and was restored.
  The contract states the limits the review listed ("Review of 2026-11-07" bullet).
- Gates on the final tree:
  test 678 run,
  678 passed,
  18 skipped (`--no-fail-fast`);
  `lint` and `lint:clippy` with warnings denied exit 0.
- A separate defect,
  not cargo-shadow's:
  one test run hung for more than twelve minutes
  because two `search-shadow` links of different versions on `PATH` (main's `.cache/search-shadow-bin`,
  0.2.2,
  and a worktree's stale one,
  0.2.1) each chose the other as the real `grep` and re-executed in a loop at 89 percent CPU
  (`strace -e execve`:
  alternating `execve` of the two `grep` paths in one process id).
  Refreshing the worktree's links through `mise run toolchain:install` ended it;
  the same test then passed in 0.14 s.
  This is the owner's "never invoke ourselves recursively" and "nothing should ever wait forever" for `search-shadow`:
  its skip of itself does not hold across two versions.

Still open from the resume point (not done in this merge,
in the order the sixth pass asked):
group 5 (path-targeted output barrier failure,
refused output with `CompletionSync` failure,
independent syscall observation;
the review found the new journal-barrier test passes with the real directory syncs replaced by `Ok(())`),
group 8,
group 9 K2 retry,
groups 10 and 11,
group 12,
group 13,
group 14,
the group 1 wording,
the fifth-pass open list,
and rebuilding the packets as cert7 under 250 KB each with the manifest.

## Sixth-pass follow-up (worktree `pi-cargo-shadow-sixth-pass`, branch `fix/cargo-shadow-sixth-pass-review-items`)

Started 2026-11-08T15:42:46-05:00 from the `Still open from the resume point` list,
in the order the sixth pass asked.
Each item was checked against the code first;
the ones already covered in round 21 are named as such.
Every new test was shown red under a planted change of the production line,
and the plant was restored (with a file copy inside this worktree,
never a Git restore).
The whole suite and the lint were run on this tree (see the Gates section).

### Group 5 (output barriers)

Done:

- A path-targeted barrier switch joins the failpoints (`arm_output_file_sync_of`,
  `arm_output_directory_sync_of`,
  `output_file_sync_of`,
  `output_directory_sync_of`,
  `path_targets_armed`),
  and `make_durable` takes the real sync's result.
- `a_manifest_barrier_that_fails_on_its_own_stops_the_ordinary_finish` fails the manifest's file barrier and its
  directory barrier separately;
  the lockfile's barrier succeeds first,
  the fault is consumed,
  no marker or cleanup follows,
  and the journal stays active.
  Red when the ordinary finish ignores the manifest's barrier error.
- `a_recovery_manifest_barrier_that_fails_on_its_own_is_mandatory_and_keeps_the_journal` does the same through
  `recover_report` for a restored root;
  the report is mandatory,
  names the manifest,
  and keeps the journal,
  the settled lockfile and the restored manifest.
  Red when recovery skips the manifest barrier.
- `a_refusal_survives_a_failed_completion_directory_sync_and_names_it` adds the refused outcome with
  `CompletionSync` (the rename succeeds,
  the parent barrier fails) to the rename case.
  Red when the install step drops the completion error from a refusal.
- `the_completion_parent_barrier_is_synchronized_by_the_kernel_before_the_journal_is_removed` runs
  `finish_journal` in a helper under `strace` and requires the fsync of the directory that lists the marker
  between the rename and the removal.
  Red when `sync_marker_parent` keeps its trace and failpoint but drops `sync_directory`.

### Group 8 (recovery and preservation)

Done:

- `a_preferred_link_failure_whose_fallback_barrier_fails_keeps_the_stranded_temporary_and_both_names`:
  the preferred kept directory refuses the link,
  the fallback link succeeds,
  its barrier fails,
  and both the fallback name and the stranded exchange temporary keep the inode and are named in the message.
  Red when the durability failure path unlinks the stranded temporary.
- `the_recovery_makes_a_kept_roots_lockfile_and_manifest_durable_through_the_real_barrier` now also pins the recovery's
  order (both output barriers,
  marker,
  marker parent,
  removal);
  `an_unready_journal_reports_its_mandatory_and_cleanup_failures_through_the_report` adds the real
  `OutputFileSync` and `OutputDirectorySync` faults to the unready schedule,
  beside the synthetic `RecoverDurability` one.
  Red when the recovery's lockfile `make_durable` loop is removed.
- `assert_recovery_keeps_a_damaged_ready_journal` now asserts the manifest too,
  so a recovery that restores manifests before rejecting a damaged ready record is red;
  it was shown red with such a
  plant.
- `an_update_stops_before_any_work_when_a_mandatory_recovery_finish_fails` covers the update call site:
  the journal (renamed `.done` by the successful rename,
  with the failed parent barrier) is kept,
  its snapshot is unchanged,
  and cargo was never asked.
  Red when the update call site's mandatory guard is removed.
- `a_restore_that_fails_after_the_exchange_keeps_the_edited_manifest_and_the_journal`:
  a target-armed fault (`arm_keep_link_of`) fails the preservation of the file the exchange displaced,
  after the exchange put the snapshot live.
  The root is treated as failed,
  its edited manifest is untouched,
  the journal and its snapshot are kept,
  the displaced inode keeps the stranded exchange temporary,
  and a control without the fault restores both.
  Red when the recovery treats the failed root as restored.
- `a_failing_exchange_or_a_read_only_link_keeps_both_inodes_named` and
  `a_filesystem_without_exchange_fails_the_restore_and_overwrites_nothing` count their swap callbacks;
  the second was shown red when the passed swap is bypassed.

Still open for group 8:

- the preferred-directory failure matrix for the queued errors (`ENOSPC`,
  `EXDEV`) at the fallback preservation and barrier stages,
  not only at the preferred destination.

### Group 9 (K2 retry)

Done:
`a_directory_whose_own_or_parent_barrier_failed_is_not_taken_as_durable_by_a_later_preservation` retries a
preservation after the new directory's own barrier failed and after its parent's barrier failed,
and requires the walk through the remembered directory to the anchor.
Red for both cases with `remember_unsynced` removed.

### Groups 10 and 11 (boot identity, views, member admission)

Done:

- `a_boot_identity_that_is_not_a_reboot_keeps_recovery_and_admission_refused`:
  another machine,
  an unreadable current identity,
  a malformed current identity,
  a malformed recorded identity and a record without an identity,
  each through `recover_report` and `begin`,
  with the live output,
  journal and snapshot checked,
  and the well-formed record of another boot as the control.
  Red with the machine-id comparison removed and with the well-formedness pair removed.
- `a_journal_seen_through_another_view_is_refused_and_kept` now also refuses admission for each altered journal view
  and keeps the journal set and the snapshot;
  `a_recovery_in_another_mount_namespace_refuses_and_keeps_the_journal` asks admission inside the namespace and requires
  the cgroup-parent reason.
- `a_new_transaction_does_not_start_beside_a_scope_with_a_member_until_it_ends` keeps and checks the journal,
  its record and its snapshot between the first refusal and the scope's end,
  and refuses a second admission while the member lives.
- `a_survivor_of_one_journal_stops_the_recovery_of_another` and
  `a_new_transaction_does_not_start_beside_a_survivor` acknowledge their surviving record,
  so the live member is the only blocker,
  and settle after the member ends as the control;
  `a_malformed_group_record_defers_the_recovery` requires the malformed-line diagnostic and the retained snapshot,
  and settles after the acknowledgement.
  Red with the live process-group check removed.

### Group 12 (preparation and skips)

Done:

- `a_wrapper_killed_between_the_launcher_spawn_and_its_record_is_recovered_conservatively` adds a test-only
  after-launch hook (between `spawn` of the scope's launcher and the launcher record) and kills a real wrapper there:
  the journal holds a scope record and no launcher line,
  recovery refuses because an absent scope does not show the launcher finished entering it,
  keeps the journal and the live output,
  and the acknowledged journal settles as the control.
  Red with the unsettled-scope rule removed.
- the delegated-scope skip now says it is not evidence for the schedule and names the variable to unset.

### Group 13 (weaker path and acknowledgement)

Done:

- `a_record_that_is_written_only_in_part_or_cannot_be_written_or_synced_starts_no_program` covers record write,
  short write and record sync with the weaker route forced,
  no payload started,
  the record step traced and no launch,
  and the control shows attempt,
  write,
  sync,
  launch in order.
  The fsync itself is guarded by the existing
  `the_record_sync_is_given_to_the_kernel_after_the_write`,
  which was shown red when the real `fsync` was replaced.
- the acknowledgement writer and reader now take the real operations' results
  (`acknowledge`'s directory sync,
  `is_acknowledged`'s file and directory syncs);
  `a_reader_that_cannot_make_an_acknowledgement_durable_blocks_recovery_and_admission` drives both consumers for each
  reader fault,
  and `the_acknowledgement_barriers_are_the_real_syscalls_in_writer_and_reader_order` observes the writer's publication
  followed by a directory fsync and the reader's file-before-directory order under `strace`.
  The writer has two directory barriers
  (`atomic_write` already syncs the directory,
  and `acknowledge` syncs it again),
  so no single removal is observable;
  removing both was shown red.
- `the_acknowledge_command_works_as_printed_with_a_quoted_path_and_refuses_while_the_workspace_is_locked` runs the printed
  repair's quoted word through `/bin/sh`,
  takes the acknowledgement command through `run_cli` for a path with a space,
  and shows the command refuses while the workspace lock is held.
  The hint now shell-quotes the path.
  Red when the quote is dropped.
- `an_acknowledgement_resolves_a_malformed_an_unreadable_and_another_views_journal` gives the positive controls for the
  three broader uncertainty classes.

Still open for group 13:

- a CLI-level test that the command itself (not only the library) makes the marker durable,
  beyond the library's syscall observation.

### Group 14 (launcher)

Done:

- the launcher fixture's append-only entry trace now records the invocation depth and the starting shadow,
  and `an_unrecorded_launcher_that_leads_back_stops_the_second_start_at_once_and_its_message_fixes_the_problem_when_copied`
  requires every entry to be at the first shadow's depth,
  so no second shadow re-enters a launcher and each second shadow is terminal in its chain.
  Red with the arrival stop's return removed (depths 1 to 4 appeared).
- the stale bounded-fallback test reference is removed from this plan and from `doc/planning/cargo-shadow-guarantees.md`,
  which now names the current tests.

Still open for group 14:

- tests at the production start sites with a handshake between selection and check.
  Round 21 covers the window after the check (`after_check_for_test`) at `Vetted::command`,
  which both shim start sites use,
  but no test runs a selection,
  then swaps the path,
  then lets the shim's own `vet` run,
  so removing `vet` from both shim sites and from `RealCargo::run` is not yet shown red.

### Group 1 wording and the fifth-pass open list

- `assert_scope_record` is now described as proving the record,
  not placement (`doc/planning/cargo-shadow-guarantees.md`,
  "What the map does not claim").
- the double-fork probe now says it has no reparenting handshake.
- the unreadable-record case also refuses admission and keeps the journal and snapshot.
- R2 stranded path and the acknowledgement writer's ordering are already covered.

Still open from the fifth-pass list:

- the mandatory-finish refusal through a real shim run;
- the blocked-feeder handshake of the input-ignoring timeout test;
- the start-time error line naming the namespace;
- the `missing_entry_verdict` and `start_of` consumer wiring;
- the group 1 refusal when the emptiness proof fails;
- the post-exchange group 8 test named above;
- rebuilding the packets as cert7 under 250 KB each with the manifest (skipped in this session:
  the packets were for a reviewer).

### Gates

- `mise run //package/cargo-shadow/cli:test:debug`:
  800 tests run,
  800 passed,
  21 skipped,
  exit 0 (the skips are the host-gated schedules the tests name).
- `mise run //package/cargo-shadow/cli:lint:clippy` (release,
  all targets,
  warnings denied):
  exit 0.
- The release `test` gate was started and stopped after ten minutes with 73 of 800 tests passed:
  the host carried a load average of about 125 on 8 CPUs from other sessions' suites,
  and two scope tests had been running for five minutes each.
  It is not reported as passing or failing;
  the `mise run //package/cargo-shadow/cli:test:debug` task is the run this session reports,
  and the release gate remains open.
- Nothing was released,
  locked or installed,
  and nothing was merged into main.

### Auto-push

Commits of this branch were saved locally.
The shared repository config had `remote.origin.pushurl = DISABLED`,
so the commit service could not push;
the owner reported that this was undone by removing the setting.
No agent of this session changed the Git config.
