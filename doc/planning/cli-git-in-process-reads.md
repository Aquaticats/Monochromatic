# Cutting read-only Git starts with in-process reads

Status:
implemented on the branch that carries this file;
measured as below.
Owner of the work:
the read-reduction agent (`abdf738d7cb09a5e6`),
for `package/git-policy/cli`.

## Requirements

The owner said,
and these are the requirements:

> Invoke real git as rarely as possible.

> git wrapper should embed in forbidden-strings and the like.
> That's why we rewrote everything in Rust in the first place.

> Please move faster.

The work is independent of the commit architecture.
Both the thin client and the resident single-writer service need cheap read-only repository facts,
and write paths (`update-ref`,
`reset`,
`write-tree`,
`pack-objects`,
`index-pack`,
`add`) stay Git.
Locks,
owner records,
the shadow repository,
replay,
reservations and journals are untouched.

## Evidence that was verified

The design review (`git show 9c581b7e21:doc/planning/cli-git-design-review.md`) counted 27 read-only Git starts of the 39
the budget harness allows per commit.
One explicit-path commit through the wrapper,
traced with `strace --follow-forks` on this branch before any change,
started 38 Git processes;
the budget table of `src/native/binary_budget_tests.rs` agrees (38 entries for the plain scenario).
Of those,
27 are read-only:
`cat-file` 3,
`config` 1,
`diff` 1,
`diff-index` 4,
`diff-tree` 1,
`for-each-ref` 1,
`ls-files` 4,
`remote` 1,
`rev-parse` 8,
`show` 1,
`symbolic-ref` 1 and `version` 1.
Three of those (`rev-parse --verify HEAD`,
`cat-file commit`,
`symbolic-ref`) run against the shadow repository,
which the pending architecture replaces,
so they were left alone.

The same trace method on common forwarded commands showed the larger finding:
every wrapped command (`status`,
`log`,
`diff`,
`branch`,
`show`,
`fetch`,
`push`,
`ls-files`) started the location
`rev-parse` (bare flag,
Git directory,
common directory,
top level,
prefix) before the command itself.
That query is one start on every invocation,
which is why it comes first in the ranking.

## Ranking

By starts removed times frequency:

1.  The location query:
    one start on every wrapped command,
    and the same file-system walk gives the Git paths
    (`--git-path index`,
    the transaction layout,
    the capture layout) that cost three more starts per commit.
2.  The object and index readers:
    `ls-files` 4,
    `diff-index` 4,
    `diff` 1,
    `diff-tree` 1,
    `show` 1 per commit.
3.  Refs and configuration:
    `config` 1,
    `for-each-ref` 1,
    `rev-parse --verify` 2,
    `@{upstream}` 1,
    `remote get-url` 1.
4.  `cat-file --batch` fed object names in the transaction runner.

Left as Git:
`version` (already cached after the first run),
the shadow-repository reads,
the long-lived candidate
object reader (`cat-file --batch` started by `candidate_reader.rs`,
owned by the commit-path agent),
and one `rev-parse --git-path index` that runs with a private index.

## What was built

All files are `src/native/repo_read_*.rs`,
each with a differential test file beside it.

- `repo_read_config_text`:
  a configuration text parser that returns nothing for any spelling outside the plain grammar.
- `repo_read_config_scan`,
  `repo_read_config_list`:
  scope files,
  include following,
  the merged `config --list` listing.
- `repo_read_layout`:
  the location,
  Git paths and object format,
  with the checks the Declined cases paragraph lists.
- `repo_read_object`,
  `repo_read_pack`,
  `repo_read_odb`:
  loose objects,
  version 2 pack indexes,
  deltas,
  alternates.
- `repo_read_tree`,
  `repo_read_index`,
  `repo_read_diff`:
  trees,
  commits,
  the index and the sorted-list difference.
- `repo_read_refs`:
  loose and packed refs,
  symbolic chains,
  the full list.
- `repo_read_query`,
  `repo_read_query_refs`:
  the exact command lines that are answered,
  with standard output,
  standard error and exit code.
- `repo_read_hook`:
  the three runner entry points that try the answers first
  (`run_metadata_git`,
  transaction `run_git`,
  the auto-push query runner).

The rule for all of them is the same:
answer only after proving the answer equals Git's,
otherwise return nothing and
start Git.
Declined cases include `GIT_*` variables that can move a path,
global options other than `-C`,
bare repositories,
links,
other file systems,
reftable,
conditional includes,
unmerged or intent-to-add entries,
a `config.worktree` that makes the repository bare,
moves the worktree or holds an unprovable value
(otherwise read,
as Git reads it,
when `extensions.worktreeConfig` is on),
a file that became a directory,
gitlinks,
a possible rename in the porcelain `diff`,
merge commits,
message encodings,
replace refs,
shallow or grafted history,
split and sparse indexes,
ambiguous short names and broken refs.
`CLI_GIT_IN_PROCESS_READS=off` turns every answer off.

## Technology choice

The repository's technology-choice discipline asks for the lockfile first.
The crate's `Cargo.lock` carries `gix-index`,
`gix-object`,
`gix-hash`,
`gix-features`,
`gix-path`,
`gix-date`,
`gix-actor` and `gix-validate` (through the scanner),
and no `gix-odb`,
`gix-pack`,
`gix-ref`,
`gix-config`,
and no zlib implementation at all (`flate2`,
`miniz_oxide` and `zlib-rs` are absent).
Loose objects and packed entries are zlib streams,
so inflate is the one real dependency question.
The candidates came from the lockfile and the registry cache only,
so the broad-discovery check-in does not apply;
a full vet report with a registry query schedule was not written,
which is a limit of this work.

- gix sub-crates (`gix-odb`,
  `gix-ref`,
  `gix-config`,
  `gix-discover`):
  the lockfile grew from 94 to 128 packages
  in an offline stub that adds them to what the lockfile has (measured with `cargo metadata --offline` on two stub
  crates),
  that is 34 more packages,
  with the zlib backend among them.
  The binary and the build were not measured for this option.
- Hand-written readers with `miniz_oxide` for inflate (chosen):
  2 more packages (`miniz_oxide` 0.9.1 and `adler2`
  2.0.1),
  pure Rust,
  one `decompress_to_vec_zlib_with_limit` call site in this crate.
  Its source was not audited for this choice;
  the later audit is
  `doc/audit/tech-miniz-oxide-0-9-1-and-adler2-2-0-1-vet-2026-11-01.md` (no gate failed).
- Hand-written readers with a hand-written inflate:
  no packages,
  but a decoder that is ours to keep correct for
  all three block types and for corrupt input;
  not chosen because a maintained decoder exists and costs 2 packages.
- Loose objects only,
  ask Git for packed ones:
  no packages,
  but packed objects are the common case in any repository
  that ran maintenance,
  so most commits would still start the processes this work removes.

Ranking:
miniz_oxide readers > hand-written inflate > gix sub-crates > loose only.
The first beats the second because the decoder is maintained and tested upstream,
at a cost of two packages.
The second beats the third because 34 packages is far larger than the readers' own surface and the readers are needed
anyway for the decline rules.
The third beats the fourth because it removes the starts the fourth keeps.

## Measurements

Binary size of the release executable:
8,707,536 bytes before,
8,887,064 bytes after,
179,528 bytes more (2.1 percent).

Release build time:
the incremental release build of the crate took 24 seconds after the change.
The same step before the change was not timed on its own (the baseline run built three crates in 58.8 seconds),
so no before and after comparison of build time is claimed.

Latency,
from `mise run //package/git-policy/cli:native:perf` (`bin/native-perf.ts`,
30 samples per scenario,
paired with
direct Git in each run),
run as alternating arms on a loaded host:
baseline binary A and new binary B,
five runs of A and three completed runs of B (one B run stopped in the script's
own cleanup with `ENOTEMPTY` on the pack directory of its scratch repository,
a race with the repository's
background maintenance that is not caused by the readers;
fixed,
see "Follow-up findings").

- Added latency of a forwarded `rev-parse`,
  medians per run:
  A 1.7,
  1.7,
  1.6,
  1.9,
  1.8 ms;
  B 0.9,
  1.1,
  0.9 ms.
  The spread inside the A arm (same binary,
  five runs) is 0.3 ms,
  and inside the B arm 0.2 ms,
  both smaller than the 0.8 ms difference between arms.
- Added latency of a forwarded `status`:
  A 1.7,
  1.7,
  1.8,
  1.8 ms;
  B 0.9,
  1.1,
  0.9 ms.
- Added latency of an explicit-path commit:
  A 104.4,
  102.8,
  105.8,
  104.6,
  99.4 ms (range 6.4 ms);
  B 70.2,
  73.1,
  71.3 ms
  (range 2.9 ms).
  The commit is about 33 ms faster,
  with the arms far apart relative to the range inside each.

Git start counts per budget scenario (`src/native/binary_budget_tests.rs`,
exact counts,
strace of the process tree).
Before is the table at the start of this work,
after is the table now;
the configured-hook scenario's after value also
reflects the hooks removal that landed on main in the meantime.

- Plain commit:
  38 Git starts before,
  18 after;
  read-only starts 27 before,
  7 after.
- Commit with one configured hook:
  43 before,
  18 after (the hook starts are gone on main);
  read-only 28 before,
  7 after.
- Amending commit:
  37 before,
  17 after;
  read-only 27 before,
  7 after.
- Second commit of the repository (warm):
  40 before,
  20 after;
  read-only 26 before,
  6 after.

The remaining read-only starts are `cat-file` 3,
`rev-parse` 2,
`symbolic-ref` 1 (shadow) and `version` 1 (cold cache).

## Controls and what they did not exercise

Every reader has a differential control that runs real Git 2.56.0 and the reader over the same repositories and
compares standard output,
standard error and exit code,
and a control for every decline rule.
Covered:
loose and packed objects with deltas,
SHA-1 and SHA-256,
annotated and lightweight tags,
packed refs with
loose overrides,
symbolic refs,
linked worktrees (location read,
Git paths and refs declined),
a separate Git
directory,
alternates,
replace refs,
shallow and graft files,
reftable (declined),
split index (declined),
index versions 2 to 4,
intent-to-add,
unmerged paths,
gitlinks,
file-to-directory changes,
renames,
merge commits,
non-UTF-8 and very long paths,
symbolic-link start directories,
include files and injected command entries.

Not exercised,
and so not claimed:
partial clones with promised objects (a missing object declines to Git,
but no
fixture has a promisor remote),
trees with millions of entries,
an owner mismatch on the Git directory
(the check is in the reader and has no control because the controls cannot change a file's owner),
a device boundary on the walk up to `.git`,
the index checksum (not verified;
Git verifies it),
and Windows (the readers are Unix-only and every query starts Git there).

## Independent review

A static review by a second model (`openai/gpt-6.1-sol`,
no tools,
a 240 KB diff of the readers;
it could not run Git)
reported 23 answer-equivalence findings and several coverage holes.
Fixed in follow-up commits,
each with a control:
last assignment wins for `GIT_INDEX_FILE` and `GIT_OBJECT_DIRECTORY`;
bounded reads of `info/alternates` and `packed-refs`;
the pack size checked before a window is allocated;
every pathspec checked after `:/`;
environment-supplied configuration entries checked for missing values and for the
content keys;
`GIT_CONFIG_NOSYSTEM` read as a boolean;
relative scope file paths declined;
`diff.relative` declined;
symbolic chains (`HEAD` through an alias branch) declined;
a second `branch.<name>.merge` declined;
a dangling
symbolic tracking ref declined;
the bare-name lookup rule of `@{upstream}` abbreviation;
sequential `-C` validation;
repeated `--diff-filter` declined;
`objects` and `refs` required to be directories;
the pack header checked;
reserved index flags and non-canonical tree and index modes declined;
misplaced peeled lines in `packed-refs`
declined;
NUL bytes in values and messages declined;
`GIT_TRACE*` no longer accepted;
SHA-256 `REF_DELTA` headers
readable;
synthesized failures only under an English locale.

Left open by that review and fixed afterwards:
Git's typed validation of configuration values (a non-boolean
`core.fileMode` made Git fail where the readers answered),
the assumed `/etc/gitconfig`,
and the sparse-index control
that did not use a real sparse index.
See the sections that follow.

Guard controls were shown failing,
one guard at a time on a restored tree:
disabling the discovery-key check,
the file-to-directory check,
the rename guard of the porcelain `diff`,
the
unmerged and intent-to-add check,
the ambiguous upstream check,
the replace-ref and shallow check,
and the switch each
made its matching control fail.
Disabling the reftable directory check alone did not fail a control,
because reftable repositories are also declined by
their `extensions.refstorage` key;
that check is a second guard on the same case.

## Typed configuration values

Real Git dies,
or warns,
at startup on a value that does not fit the key's type,
for keys the readers never look at
(`core.fileMode = maybe` gives `fatal: bad boolean config value 'maybe' for 'core.filemode'`).
The readers answered in such repositories.

Method:
every name `git help --config` lists in Git 2.56.0 plus every name in `Documentation/config/*.adoc`
(about a thousand),
each set in the repository's configuration to seven spellings (a word,
a bare name,
an empty value,
`-1`,
`0`,
`3`,
an overflowing number),
and thirteen command lines run for each (the eleven reader shapes,
the location
query and `cat-file --batch`),
comparing exit code,
standard error and the size of standard output with an unchanged
repository.
Git's source was read for the keys that fail (`environment.c` `git_default_core_config` and its helpers,
`repo-settings.c`,
`config.c`);
the tarball of tag v2.56.0 stood in for a clone,
because the sandbox refuses
`gh repo clone`.

Result:
about 240 keys change some command (about 45 of them `advice.*`).
They fall into booleans (all of `advice.*`,
most of `core.*`,
`diff.*`,
`log.*`,
`grep.*`,
`pack.*`,
`remote.<name>.*`),
integers (with ranges:
`core.abbrev` from 4,
compression levels from
`-1` to 9,
a leading zero is octal in Git),
enumerations (`push.default`,
`core.checkStat`,
`diff.algorithm`,
`transfer.credentialsInUrl`,
and others),
colors,
paths (`~user` of an unknown user dies),
text keys that die when bare
(`user.name`,
`remote.<name>.url`),
and keys that only warn (`core.fsync`,
`core.fsyncObjectFiles`,
`diff.dirstat`).
Two surprises:
a bare `remote.<name>.tagOpt` kills `git remote get-url` with a signal,
and a bare
`core.repositoryFormatVersion` was treated by the layout reader as absent.

Design:
`src/native/repo_read_config_typed.rs` holds the key table and a validator per kind that accepts only
spellings it can show Git accepts silently (decimal numbers of at most nine digits,
no octal,
words in Git's lists).
Every other spelling of a typed key makes the layout reader decline,
and with it every query,
because all of them start
from the layout.
The proof walks the repository file,
the system and global files,
their includes (conditional ones
too,
without evaluating the condition) and `GIT_CONFIG_COUNT` entries.
Declining costs one Git start in a repository that is misconfigured or uses a spelling outside the subset
(for example `core.packedGitLimit = 256m` is accepted,
an octal or hexadecimal number is not).

Controls (`repo_read_config_typed_tests.rs`):
representative rejected values must be declined and must change Git's
output;
every spelling the table accepts must leave Git silent on all shapes and keep the readers answering;
and a
sweep over every key `git help --config` lists with the seven spellings must decline or equal Git,
in batches that are
split down to the key when they differ,
so a Git upgrade that adds a typed key is noticed.
The sweep takes under 20 seconds in a debug build.
Without the proof,
the sweep reports every typed key and the rejected-values control fails on `core.fileMode`.
Also fixed from the sweep:
`remote.<name>.fetch` and `remote.<name>.push` are checked as plain refspecs (Git fails with
`invalid refspec`).

### `core.fsmonitor`

The key is the switch of the built-in monitor daemon or the path of a hook program Git runs
(`fsmonitor-settings.c`,
`lookup_fsmonitor_settings`:
a boolean value turns the daemon on or off,
any other value is a hook path).
The table first accepted only false spellings,
because a test that sets the key to a true word starts a daemon.
Every user with `core.fsmonitor = true` in their own file then made the layout reader decline every repository,
so each commit paid thirteen more Git starts (`rev-parse` 5,
`diff-index` 3,
`ls-files` 2,
`config`,
`diff` and `diff-tree` 1 each;
measured with `binary_budget_tests.rs` on Git 2.56.0).
Real Git starts the daemon for the shapes that read the index,
and the true word changes neither their output nor their exit status
(compared on every shape in `the_query_readers_equal_git_under_a_user_level_fsmonitor_true`,
with the daemon running and an index
that carries its extension),
so the table now accepts the true words `true`,
`yes` and `on` with the false spellings.
A hook path,
a number other than zero and the bare name still decline:
Git runs a hook for commands that read the index,
and reads the bare name and the numbers as true,
so declining them is conservative.
Starting the monitor can fail,
and real Git 2.56.0 says nothing about it on standard error or in the exit status
(a directory where the socket goes,
`the_query_readers_equal_git_when_the_monitor_cannot_start`).
`fsmonitor.allowRemote` and `fsmonitor.socketDir`,
which Git reads once the monitor is on,
are typed.
Accepted limit:
a listener of the same user on the monitor's socket that answers with a malformed reply would make Git print a
protocol error the readers do not,
and the wrapper does not decline monitor-contacting queries for it,
because the owner decided the wrapper leaves Git's monitor alone (2026-11-02:
"Why are we messing with git's fsmonitor?
I already told you it's fine for git to run it.").
Configuration-enabled Trace2 is declined:
an active `trace2.normalTarget`,
`trace2.eventTarget` or `trace2.perfTarget` makes Git write trace output the readers do not.
An index with the `FSMN` and `UNTR` extensions is read like any other (`parse_index` skips optional extensions).

## System configuration path

The readers assumed `/etc/gitconfig`.
Git compiles `ETC_GITCONFIG` in (`Makefile` sets `sysconfdir` to `/etc` only when
the prefix is `/usr`;
otherwise it is relative to the prefix,
resolved against the executable in a runtime-prefix build).
`git var GIT_CONFIG_SYSTEM` prints the path the executable reads (`builtin/var.c`,
`git_config_val_system`).
`src/native/git_system_config.rs` asks the real Git once per executable identity (path,
size,
times,
device,
inode,
the same identity as the `git version` cache) with an empty environment and stores the answer beside the version cache
(`git-system-config-<hash>`).
The real Git is the one the forward-target variable names.
When it is unknown,
the file is
unknowable and the readers decline,
unless `GIT_CONFIG_NOSYSTEM` or `GIT_CONFIG_SYSTEM` settle it.

Verified with a Git built from the v2.56.0 tarball with prefix `<scratch>/opt-git`:
`git var GIT_CONFIG_SYSTEM` prints
`<scratch>/opt-git/etc/gitconfig` while `/usr/bin/git` prints `/etc/gitconfig`.
The committed controls use the host Git
(equal to what it prints for itself,
asked once,
a replaced executable asked again) and a stand-in answer for another
prefix;
the built Git itself is not part of the controls because it cannot be shipped with the repository.

## Follow-up findings

- A real cone-mode sparse index (`sparse-checkout init --cone --sparse-index`) is declined,
  with and without the
  configuration keys that announce it;
  the decisive guards are the required `sdir` extension and the non-canonical
  directory mode,
  and removing both makes the control fail with a listing of a sparse directory.
- A loose object with bytes after its zlib stream was answered;
  Git says `garbage at end of loose object`.
  Fixed with the consumed length from the decoder's low-level function.
- The decoder's cost for chained empty blocks grows with the stream's length,
  not its output (upstream issue 198);
  a stream longer than 512 KiB of compressed bytes now goes to Git.
- A panic inside a reader is caught at the hook and Git answers instead.
- `native:perf` removed its scratch tree without waiting for processes Git detaches:
  `strace --follow-forks` shows every
  `git commit` starting `git maintenance run --auto --quiet --detach`,
  which daemonizes out of the harness's process
  tree and pipes,
  so the harness's close event does not cover it.
  The writer of the one failed run was not captured,
  so the exact process is an inference;
  the race itself is reproduced by a detached writer.
  Cleanup now waits for every process that has the tree as its working directory,
  its executable or an open file,
  up to a deadline,
  and fails naming the process when one remains
  (`bin/native-perf-scratch.ts`;
  the control reproduces `ENOTEMPTY` on the `pack` directory without it).
