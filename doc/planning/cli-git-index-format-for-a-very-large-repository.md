# The index of a very large repository, and what four Git settings cost the commit path

Written 2026-11-10T19:07:00-05:00,
on branch `perf/git-policy-cli-index-format-for-a-very-large-repository`,
which starts at `main`'s `e498914475`.

The owner's bounds that decide this question:
"commits vs native git should not take 5s longer",
"Commits should never fail",
"Prioritize fixing the jams and release jam fixes asap".
Every commit's staging steps and status checks read and write this repository's index,
privately and in the real file,
so what the index costs is part of what every commit costs.

## What the repository holds

- 23 082 tracked files (`git ls-files | wc -l`).
- This worktree's own index file,
  read only at 2026-11-10T18:50:53-05:00:
  3 193 706 bytes,
  file format version 2,
  with the `TREE` (cache-tree) and `UNTR` (untracked cache) extensions and a hashed trailer.
  It held 3 194 706 bytes at 2026-11-10T17:29:11-05:00,
  with the same version and extensions.
  The untracked cache is in this repository's index already,
  because the person's global configuration sets `core.untrackedCache`;
  a clone made under a home of its own writes no such extension.
- The same bytes written as file format version 4 by Git 2.56.0 in a scratch copy of the clone
  (`git update-index --index-version=4`,
  2026-11-10T18:50:53-05:00):
  2 012 226 bytes,
  which is 1 181 480 bytes and 37.0 percent less.
- The volume is btrfs on a SATA SSD,
  measured 99 percent busy while adds ran.

## The four settings, and what Git's own documentation says

Read from `git-config` of the installed Git 2.56.0 at 2026-11-10T18:40:00-05:00:

- `index.version`
  "Specify the version with which new index files should be initialized.
  This does not affect existing repositories."
  So it decides the format of an index file Git creates from nothing,
  and an existing file keeps the version it was written with.
  Measured:
  with `index.version=4` set and a refresh and a status that both wrote the file,
  an existing version 2 index stayed version 2 in bytes;
  `git update-index --index-version=4` is what converts one.
- `core.untrackedCache`
  "It will be kept,
  if this variable is unset or set to keep.
  It will automatically be added if set to true.
  And it will automatically be removed,
  if set to false."
  `keep` by default unless `feature.manyFiles` is on.
- `feature.manyFiles`
  three defaults:
  `index.skipHash=true`,
  `index.version=4`,
  `core.untrackedCache=true`;
  for the version,
  "This does not affect existing repositories".
  Measured in the fixtures:
  a repository with only `feature.manyFiles=true` wrote a version 4 index file
  carrying an untracked cache and a trailer of zeros,
  which is all three settings at once.
- `index.skipHash`
  "do not compute the trailing hash for the index file.
  This accelerates Git commands that manipulate the index,
  such as git add,
  git commit,
  or git status.
  Instead of storing the checksum,
  write a trailing set of bytes with value zero".
  Clients older than 2.13.0 refuse such an index,
  and clients older than 2.40.0 report a corrupted index in `git fsck`.

Measured on Git 2.56.0 in a throwaway repository at 2026-11-10T18:39:00-05:00,
because the wrapper writes index files itself:
an index whose trailer is zeros is read with no complaint without `index.skipHash`,
and an index with a real checksum is read with no complaint with `index.skipHash=true`,
after which the next Git write stores zeros again.
So either trailer reads here,
and only byte-for-byte equality with Git's own write decides which one the wrapper should write.

## How it was measured

Every probe ran in a `git clone --local --no-hardlinks` of this worktree at `e498914475`
under `${HOME}/temp/agent/ixi`,
with its remote removed,
its own `HOME` and `XDG_RUNTIME_DIR`,
`GIT_CONFIG_NOSYSTEM=1` and `GIT_CONFIG_GLOBAL=/nonexistent-global-config`,
so no configuration of the person's answers for anything.
Two facts shaped the fixture:

- Its first clone was made with the person's global configuration,
  whose `core.untrackedCache` had already given the index an `UNTR` extension and whose git-lfs filter had smudged
  1 578 files,
  so the index and the worktree did not agree under a clean home.
  The fixture was rebuilt from a clone made under the run's own home;
  its status is empty in both forms.
- It carries a build-output-shaped population of 18 000 files whose names end in `.log`,
  which this repository's `.gitignore` ignores,
  inside 300 directories,
  so `status --untracked-files=normal` has to walk directories that print nothing,
  which is what the untracked cache can remember.

Six arms,
each a reflink copy of that clone carrying one setting:

- `arm-a-baseline`:
  no setting.
- `arm-b-version4`:
  `index.version=4`,
  the file converted with `git update-index --index-version=4`.
- `arm-c-untracked`:
  `core.untrackedCache=true`.
- `arm-d-manyfiles`:
  `feature.manyFiles=true`.
- `arm-e-skiphash`:
  `index.skipHash=true`.
- `arm-f-manyfiles-skiphash`:
  `feature.manyFiles=true` and `index.skipHash=true`.

The repository policy list is emptied and committed in every arm
(`{"policies": {}}` in `monochromatic-git-policy-cli.config.jsonc`,
which stays tracked,
so no arm's status names a deleted file),
because the linked scanner and the dependent-version policy read trees and files whose cost no index setting changes,
and their cost moved the staging samples by hundreds of milliseconds between repetitions.
What is measured is the index work of the commit path,
not the policy work;
the policy work is measured in `doc/planning/cli-git-commit-latency-under-disk-pressure.md`.

Every sample calls plain `git`,
which on the sample's `PATH` is the release wrapper built from the measured branch,
as a person's own `git` is.
The wrapper starts the real Git for every forwarded command,
so a sample is the forwarded command plus the wrapper's own guards.

The operations are the ones this wrapper's commit path runs (`src/native/commit/`):
`git add -- <path>` in a private copy of the index,
`git update-index --refresh` in a private copy,
`git write-tree` in a private copy,
`git status --porcelain=v1 --untracked-files=no` (the gate's form,
`src/native/command/rule_worktree_dirty.rs`),
`git status --porcelain=v1 --untracked-files=normal` (the worktree snapshot's form,
`src/native/worktree/worktree_snapshot_source_changed_paths.rs`),
and the wrapper's own `git commit --message=<message> -- <path>` against a commit service of the arm's own.
Every private copy is made from the arm's real index outside any timing.

Method:

- Two warm-up rounds are discarded,
  then twelve measured rounds.
- Inside a round the arms alternate direction,
  so drift in the host's load reaches every arm alike.
- Every fast operation runs three times per round and the fastest sample is kept,
  so one spike of the host cannot move it;
  the wrapped commit runs once per round,
  then the run waits 1 500 ms,
  because a commit leaves work running after its answer.
- `arm-a-repeat` is the baseline arm measured a second time in every round.
  The difference between the two copies of one arm is the smallest difference this measurement can show.
- Reported are the median over the twelve rounds and,
  as the pair statistic,
  the median of the round-by-round difference against `arm-a-baseline`.

The twelve measured rounds ran from 2026-11-10T18:42:29-05:00 to 2026-11-10T18:55:46-05:00
(the commits of `arm-b-version4`,
which has one commit per round).
The host was not quiet and was not made quiet:
load average 61.8 to 139.5,
input/output pressure `some avg60` 31.8 to 43.2 percent,
measured at 2026-11-10T18:33:00-05:00,
18:50:44,
18:51:04,
18:51:47 and 18:53:20.
Every number below is a number under that pressure,
which is the state this repository's commits are in.

## What each arm's index file is

Read from the bytes at 2026-11-10T18:20:00-05:00,
before the timed rounds:

- `arm-a-baseline`:
  version 2,
  23 082 entries,
  3 098 565 bytes,
  `[TREE]`,
  hashed trailer.
- `arm-b-version4`:
  version 4,
  1 917 085 bytes,
  `[TREE]`,
  hashed trailer;
  61.9 percent of the baseline's file.
- `arm-c-untracked`:
  version 2,
  3 208 864 bytes,
  `[TREE,UNTR]`,
  hashed trailer;
  the cache costs 110 299 bytes,
  3.6 percent more.
- `arm-d-manyfiles`:
  version 4,
  2 027 384 bytes,
  `[TREE,UNTR]`,
  trailer of zeros.
- `arm-e-skiphash`:
  version 2,
  3 098 565 bytes,
  `[TREE]`,
  trailer of zeros;
  the same size,
  because the zeros are as long as the checksum.
- `arm-f-manyfiles-skiphash`:
  version 4,
  2 027 393 bytes,
  `[TREE,UNTR]`,
  trailer of zeros.

## What was measured

Medians of the twelve rounds,
in milliseconds,
with the lowest and the highest sample in parentheses.
The last line of each group is the baseline arm measured a second time,
which is the control.

`git add -- clippy.toml` in a private copy of the index:

- baseline:
  231.1 (140 to 316)
- version 4:
  222.5 (116 to 396)
- untracked cache:
  182.7 (148 to 376)
- `feature.manyFiles`:
  208.9 (121 to 1 855)
- `index.skipHash`:
  210.5 (125 to 797)
- `feature.manyFiles` and `index.skipHash`:
  165.9 (115 to 277)
- baseline again:
  212.0 (149 to 705)

`git update-index --refresh` in a private copy:

- baseline:
  68.2 (30 to 154)
- version 4:
  47.9 (32 to 87)
- untracked cache:
  51.7 (34 to 103)
- `feature.manyFiles`:
  55.6 (36 to 88)
- `index.skipHash`:
  56.8 (32 to 94)
- `feature.manyFiles` and `index.skipHash`:
  53.2 (29 to 110)
- baseline again:
  67.8 (41 to 118)

`git write-tree` in a private copy:

- baseline:
  58.7 (38 to 137)
- version 4:
  48.0 (25 to 85)
- untracked cache:
  55.4 (33 to 85)
- `feature.manyFiles`:
  35.8 (28 to 92)
- `index.skipHash`:
  55.5 (33 to 117)
- `feature.manyFiles` and `index.skipHash`:
  36.1 (25 to 63)
- baseline again:
  60.6 (34 to 109)

`git status --porcelain=v1 --untracked-files=no`,
the gate's form:

- baseline:
  42.1 (16 to 73)
- version 4:
  41.7 (17 to 83)
- untracked cache:
  33.1 (17 to 70)
- `feature.manyFiles`:
  31.4 (23 to 48)
- `index.skipHash`:
  39.1 (19 to 51)
- `feature.manyFiles` and `index.skipHash`:
  34.7 (20 to 119)
- baseline again:
  33.9 (20 to 107)

`git status --porcelain=v1 --untracked-files=normal`,
the worktree snapshot's form:

- baseline:
  356.5 (76 to 1 064)
- version 4:
  353.1 (73 to 1 887)
- untracked cache:
  46.9 (28 to 56)
- `feature.manyFiles`:
  44.5 (32 to 77)
- `index.skipHash`:
  165.3 (90 to 1 650)
- `feature.manyFiles` and `index.skipHash`:
  48.7 (30 to 102)
- baseline again:
  336.6 (85 to 1 895)

The wrapper's own commit:

- baseline:
  805.6 (389 to 3 491)
- version 4:
  683.7 (247 to 7 158)
- untracked cache:
  983.1 (253 to 3 021)
- `feature.manyFiles`:
  1 103.9 (451 to 7 753)
- `index.skipHash`:
  1 312.3 (893 to 3 643)
- `feature.manyFiles` and `index.skipHash`:
  1 157.9 (345 to 2 035)
- baseline again:
  782.3 (212 to 1 736)

The paired differences,
median of the round-by-round difference against the baseline,
in milliseconds,
with the control last:

- `git add`:
  version 4 −22.7,
  untracked cache +8.0,
  `feature.manyFiles` +2.3,
  `index.skipHash` −10.5,
  both −36.6,
  control −13.9.
- `git update-index --refresh`:
  version 4 −12.7,
  untracked cache −0.3,
  `feature.manyFiles` −2.4,
  `index.skipHash` +6.7,
  both +0.3,
  control +7.2.
- `git write-tree`:
  version 4 −18.7,
  untracked cache −7.4,
  `feature.manyFiles` −10.4,
  `index.skipHash` −7.3,
  both −18.8,
  control −3.0.
- gate status:
  version 4 +1.2,
  untracked cache −5.0,
  `feature.manyFiles` −1.6,
  `index.skipHash` −3.0,
  both −0.3,
  control +0.2.
- untracked status:
  version 4 +82.8,
  untracked cache −306.2,
  `feature.manyFiles` −306.9,
  `index.skipHash` −28.0,
  both −302.9,
  control +4.4.
- the wrapper's commit:
  version 4 +120.5,
  untracked cache +116.1,
  `feature.manyFiles` −1.8,
  `index.skipHash` +275.1,
  both +5.1,
  control −128.1.

What these numbers say:

- The untracked cache is the one setting whose saving this measurement resolves:
  306.2 ms of 356.5,
  six and a half times faster,
  on the form the worktree snapshot uses.
  The gate's own form moves by nothing beyond its control (control 0.2 ms,
  arms −5.0 to +1.2),
  because it walks no directory,
  so the cache buys this commit path nothing where the commit is gated.
- Version 4 saves 22.7 ms of 231.1 on the staging and 18.7 ms of 58.7 on `write-tree`,
  against a control of 13.9 and 3.0 ms;
  its 12.7 ms of the refresh is inside that operation's 7.2 ms control and its spread.
- Version 4 is worth more in bytes than in milliseconds:
  this repository's index is 1 181 480 bytes and 37.0 percent smaller as version 4.
  One commit writes the index into two private copies and installs it once,
  so version 4 writes about 3.5 MB less per commit,
  and every Git start that reads the index reads 1.2 MB less.
  On a volume measured 99 percent busy,
  bytes are what the disk is short of.
- `index.skipHash` alone is inside the control on every operation
  (staging −10.5 against 13.9,
  refresh +6.7 against 7.2,
  `write-tree` −7.3 against 3.0,
  gate status −3.0 against 0.2).
  Its one apparent effect,
  on the untracked status form,
  has no mechanism:
  the arms without a cache are the baseline (356.5) and `index.skipHash` (165.3),
  whose samples overlap from 76 and 90 ms upwards,
  so the difference between the two is that form's noise.
- The wrapper's own commit does not move:
  its control is 128.1 ms and the arms run from −1.8 to +275.1 ms,
  with samples from 212 to 7 753 ms.
  A setting's 20 ms cannot be seen through the queue,
  the service answer and the disk's stalls in that sample.

The wrapper's own index work,
read from the `prepareSpans` of the
`monochromatic-git-policy-cli-commit-timing.jsonl` records the wrapper writes
(last twelve commits of each arm,
medians in milliseconds):

- `index read`:
  0 in every arm.
- `index copy`:
  3.0 (baseline),
  3.0 (version 4),
  3.5 (untracked cache),
  1.5 (`feature.manyFiles`),
  1.5 (`index.skipHash`),
  1.5 (both).
- `index equals base`:
  5.5,
  5.5,
  5.0,
  6.0,
  6.0,
  5.5.
- `index stat check`:
  50.0,
  53.0,
  50.5,
  59.5,
  59.0,
  53.0.

So the wrapper's in-process index work is about 60 ms of an 800 to 1 300 ms commit,
and the stat check of 23 082 files is nearly all of it;
the file's format does not change that check,
which is why no setting moves the commit.
Whatever a setting saves,
it saves in Git's own children and in the bytes on the disk.

## What the wrapper already does with these settings

The wrapper reads and writes the index in process in two places:

- The read-only reader `src/native/repo_read/repo_read_index.rs` parses versions 2,
  3 and 4,
  keeps the cache-tree and accepts `UNTR` and `FSMN` bodies of any content,
  and declines (leaving the question to Git) an extension it does not know,
  a split or sparse index,
  and an offset table (`IEOT`) that does not name exactly the entries it parsed.
- The landing's in-process index update (`src/native/index/index_reset_file.rs`)
  reads the whole file (`index_full.rs`),
  parses versions 2 to 4,
  keeps `TREE`,
  `REUC`,
  `UNTR` and `FSMN`,
  writes the version the file had (`index_full_write.rs`),
  and decides from the configuration (`index_settings.rs`) whether it may run at all.
- Private index copies are byte copies (`src/native/commit/commit_index_bytes.rs`),
  so every setting reaches them unchanged.

Against the four settings,
before this work:

- `index.version=4`:
  handled,
  and nothing declined for it.
  The reader parses the prefix-compressed names and the writer keeps version 4
  (`an_index_with_the_monitor_and_untracked_cache_extensions_is_read` and
  `entries_equal_the_listing_in_every_version` in `src/native/repo_read/repo_read_index_tests.rs`,
  `every_index_version_equals_git` in `src/native/index/index_reset_file_tests.rs`).
- `core.untrackedCache`:
  handled where the file already holds a cache,
  which is the steady state,
  because Git keeps it and this update invalidates the entries of the paths it resets
  (`src/native/index/index_untracked.rs`).
  A configuration that would have Git *add* a cache to a file that has none declines and Git answers,
  which costs one Git process for the one commit that writes the cache.
- `feature.manyFiles`:
  declined before this work,
  so a repository that adopted Git's setting for large checkouts lost the in-process update
  and paid a `git reset --no-refresh` process per landing,
  which is the process whose 59 s under the landing's index lock is recorded in
  `doc/planning/cli-git-main-commit-within-native-plus-five.md`.
- `index.skipHash`:
  declined before this work,
  and the writer ended every file with a checked trailer,
  so the in-process update would not have written Git's file even if it had run.

What this branch changed,
with no new switch and no new configuration key
(`src/native/index/index_settings.rs`,
`src/native/index/index_full_write.rs`,
`src/native/index/index_reset_file.rs`):

- `feature.manyFiles` is resolved into the three defaults Git makes of it,
  and an explicit `core.untrackedCache` or `index.skipHash` beside it still wins,
  as Git sets those defaults only where nothing set the value.
- `index.skipHash` is read as Git reads it,
  a bare key included,
  and the writer ends the file with zeros of the checksum's own length when it is on,
  which is the file Git writes.
- The update still declines what it cannot reproduce:
  a split or sparse index,
  a sparse checkout,
  `core.ignoreCase`,
  the offset-table and end-of-index-entry settings,
  `index.threads`,
  `core.hooksPath`,
  `core.worktree`,
  and an untracked cache it would have to add.

Controls,
each of which failed before the change and passes after it:

- `index::index_settings::tests::feature_manyfiles_sets_git_s_defaults_and_explicit_keys_win`
- `index::index_settings::tests::index_skiphash_is_read_as_git_reads_it`
- `index::index_full_write::tests::a_skipped_hash_writes_zeros_where_the_checksum_would_be`
- `index::index_reset_file::tests::a_repository_with_feature_manyfiles_equals_git`,
  which compares the update's bytes with `git reset --no-refresh`'s byte for byte,
  and asserts the file is version 4 with a trailer of zeros
- `index::index_reset_file::tests::a_repository_with_a_skipped_hash_equals_git`

## What would break

- Nothing in the wrapper breaks under any of the four settings,
  before or after this change:
  a setting it cannot reproduce makes the in-process update decline and Git answers,
  so a commit never fails because of a setting.
- `index.skipHash` weakens the landing's replay guard:
  the witness of the live index file (`src/native/landing/service_land_start.rs`)
  keeps the file's last 32 bytes,
  which are the checksum,
  and under `index.skipHash` every index Git writes ends with zeros,
  so those bytes no longer tell two files apart.
  The inode,
  size,
  modification time and change time it also records still do,
  because every writer renames a new file over the index.
- `index.version=4` is not written on request by the wrapper's own writer:
  it writes the version the file had,
  which is what Git does too,
  so adopting version 4 needs `git update-index --index-version=4` once in each worktree.
- A Git older than 2.13.0 cannot read an index with a skipped hash,
  and one older than 2.40.0 reports it in `git fsck`;
  this repository pins its Git through its toolchain,
  so this is a note and not a risk here.

## Recommendation

Nothing in this section was applied:
no setting of this repository changed in this work.

1.  Adopt `index.version=4`,
    converted once per worktree with `git update-index --index-version=4`.
    It is the only setting that is worth its risk here:
    37.0 percent fewer index bytes,
    about 3.5 MB less written per commit,
    22.7 ms less on the staging and 18.7 ms less on `write-tree` per commit measured here,
    no wrapper change needed,
    no new failure path,
    and the two implementations that disagree about index bytes (Git and this wrapper's writer) are already proved
    equal through versions 2,
    3 and 4 by the differential controls.
    Its cost is one command per worktree and one more file format to keep tested.
2.  Leave `core.untrackedCache` on,
    as the person's global configuration already has it.
    It is worth the most time of any setting measured (306.2 ms on the untracked status form),
    costs 3.6 percent more index bytes,
    and its saving does not reach the gate's `--untracked-files=no` form at all,
    so on this commit path it pays only in the worktree snapshot.
3.  `feature.manyFiles` is the same thing as 1 and 2 plus `index.skipHash`,
    and this branch is what makes it free for the in-process landing update.
    Take it only if the owner prefers one key over three settings:
    it carries `index.skipHash`,
    which measured no saving and weakens the replay witness's trailer.
4.  Do not set `index.skipHash` for this repository on what was measured:
    every operation is inside the measurement's own control,
    and it trades a discriminating witness for nothing this host can see.

Ranking:
1 > 2 > 3 > 4.
1 beats 2 because it removes bytes from every write and every read of the index,
which is what the saturated volume is short of,
where 2 only saves directory walks on a form this commit path uses once,
and 2 is already in force.
2 beats 3 because 2 has no unknown in it,
while 3 adds `index.skipHash`'s weaker witness for no measured gain.
3 beats 4 because 3 is where `index.skipHash` comes bundled with the two settings that do help,
so it is the only form in which that setting should be considered at all.

## What was not done

- No setting of this repository was changed,
  by the brief's rule:
  `index.version` is still unset,
  and this worktree's index is still version 2 with 3 193 706 bytes.
- The settings were not measured against the real checkout,
  nor against the person's own commits,
  nor against the installed wrapper's service:
  the rules confine every probe to throwaway clones with a home of their own.
- The 184 s and 59 s stalls are not reproduced here;
  this record measures the steady-state work.
  No synthetic load was generated to make a stall,
  and the load that was there is reported beside every number.
- The wrapper's own commit sample cannot resolve a 20 ms difference:
  its control is 128.1 ms and its samples run from 212 to 7 753 ms.
  A measurement that could would need the stalls removed,
  which needs the queue to be idle,
  which this host is not.
- `core.fsmonitor` is not one of the four settings and was left out;
  with a monitor in force the stat check and the untracked cache behave differently,
  and `doc/planning/cli-git-in-process-reads.md` records what the wrapper does with that key.
- The in-process update still leaves a configuration that would have Git *add* an untracked cache to Git,
  because adding one means hashing directory times,
  which is a worktree walk this update does not do.
- `monochromatic-lint` over the whole package fails on files this branch does not touch
  (`src/native/service/service_server_connection_handling_tests.rs`,
  `src/native/worktree/idle_io_priority.rs`,
  `src/native/worktree/worktree_trash_hand_over.rs`,
  `src/native/worktree/worktree_trash_dispose.rs`,
  `src/native/worktree/worktree_trash_entries.rs`),
  each a `markdown/semantic-line-breaks` error or an `rust/avoidable-closure` warning;
  the files this branch changes pass it.
