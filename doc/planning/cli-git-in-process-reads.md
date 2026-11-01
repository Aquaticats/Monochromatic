# Cutting read-only Git starts with in-process reads

Status: implemented on the branch that carries this file; measured as below.
Owner of the work: the read-reduction agent (`abdf738d7cb09a5e6`), for `package/git-policy/cli`.

## Requirements

The owner said, and these are the requirements:

> Invoke real git as rarely as possible.

> git wrapper should embed in forbidden-strings and the like. That's why we rewrote everything in Rust in the first place.

> Please move faster.

The work is independent of the commit architecture.
Both the thin client and the resident single-writer service need cheap read-only repository facts,
and write paths (`update-ref`, `reset`, `write-tree`, `pack-objects`, `index-pack`, `add`) stay Git.
Locks, owner records, the shadow repository, replay, reservations and journals are untouched.

## Evidence that was verified

The design review (`git show 9c581b7e21:doc/planning/cli-git-design-review.md`) counted 27 read-only Git starts of the 39
the budget harness allows per commit.
One explicit-path commit through the wrapper, traced with `strace --follow-forks` on this branch before any change,
started 38 Git processes; the budget table of `src/native/binary_budget_tests.rs` agrees (38 entries for the plain scenario).
Of those, 27 are read-only: `cat-file` 3, `config` 1, `diff` 1, `diff-index` 4, `diff-tree` 1, `for-each-ref` 1,
`ls-files` 4, `remote` 1, `rev-parse` 8, `show` 1, `symbolic-ref` 1 and `version` 1.
Three of those (`rev-parse --verify HEAD`, `cat-file commit`, `symbolic-ref`) run against the shadow repository,
which the pending architecture replaces, so they were left alone.

The same trace method on common forwarded commands showed the larger finding:
every wrapped command (`status`, `log`, `diff`, `branch`, `show`, `fetch`, `push`, `ls-files`) started the location
`rev-parse` (bare flag, Git directory, common directory, top level, prefix) before the command itself.
That query is one start on every invocation, which is why it comes first in the ranking.

## Ranking

By starts removed times frequency:

1. The location query: one start on every wrapped command, and the same file-system walk gives the Git paths
   (`--git-path index`, the transaction layout, the capture layout) that cost three more starts per commit.
2. The object and index readers: `ls-files` 4, `diff-index` 4, `diff` 1, `diff-tree` 1, `show` 1 per commit.
3. Refs and configuration: `config` 1, `for-each-ref` 1, `rev-parse --verify` 2, `@{upstream}` 1, `remote get-url` 1.
4. `cat-file --batch` fed object names in the transaction runner.

Left as Git: `version` (already cached after the first run), the shadow-repository reads, the long-lived candidate
object reader (`cat-file --batch` started by `candidate_reader.rs`, owned by the commit-path agent),
and one `rev-parse --git-path index` that runs with a private index.

## What was built

All files are `src/native/repo_read_*.rs`, each with a differential test file beside it.

- `repo_read_config_text`: a configuration text parser that returns nothing for any spelling outside the plain grammar.
- `repo_read_config_scan`, `repo_read_config_list`: scope files, include following, the merged `config --list` listing.
- `repo_read_layout`: the location, Git paths and object format, with the checks listed below.
- `repo_read_object`, `repo_read_pack`, `repo_read_odb`: loose objects, version 2 pack indexes, deltas, alternates.
- `repo_read_tree`, `repo_read_index`, `repo_read_diff`: trees, commits, the index and the sorted-list difference.
- `repo_read_refs`: loose and packed refs, symbolic chains, the full list.
- `repo_read_query`, `repo_read_query_refs`: the exact command lines that are answered, with standard output,
  standard error and exit code.
- `repo_read_hook`: the three runner entry points that try the answers first
  (`run_metadata_git`, transaction `run_git`, the auto-push query runner).

The rule for all of them is the same: answer only after proving the answer equals Git's, otherwise return nothing and
start Git.
Declined cases include `GIT_*` variables that can move a path, global options other than `-C`, bare repositories,
links, other file systems, reftable, worktree-specific config, conditional includes, unmerged or intent-to-add entries,
a file that became a directory, gitlinks, a possible rename in the porcelain `diff`, merge commits, message encodings,
replace refs, shallow or grafted history, split and sparse indexes, ambiguous short names and broken refs.
`CLI_GIT_IN_PROCESS_READS=off` turns every answer off.

## Technology choice

The repository's technology-choice discipline asks for the lockfile first.
The crate's `Cargo.lock` carries `gix-index`, `gix-object`, `gix-hash`, `gix-features`, `gix-path`, `gix-date`,
`gix-actor` and `gix-validate` (through the scanner), and no `gix-odb`, `gix-pack`, `gix-ref`, `gix-config`,
and no zlib implementation at all (`flate2`, `miniz_oxide` and `zlib-rs` are absent).
Loose objects and packed entries are zlib streams, so inflate is the one real dependency question.
The candidates came from the lockfile and the registry cache only, so the broad-discovery check-in does not apply;
a full vet report with a registry query schedule was not written, which is a limit of this work.

- gix sub-crates (`gix-odb`, `gix-ref`, `gix-config`, `gix-discover`): the lockfile grew from 94 to 128 packages
  in an offline stub that adds them to what the lockfile has (measured with `cargo metadata --offline` on two stub
  crates), that is 34 more packages, with the zlib backend among them.
  The binary and the build were not measured for this option.
- Hand-written readers with `miniz_oxide` for inflate (chosen): 2 more packages (`miniz_oxide` 0.9.1 and `adler2`
  2.0.1), pure Rust, one `decompress_to_vec_zlib_with_limit` call site in this crate.
  Its source was not audited for this choice, which is a limit of this work.
- Hand-written readers with a hand-written inflate: no packages, but a decoder that is ours to keep correct for
  all three block types and for corrupt input; not chosen because a maintained decoder exists and costs 2 packages.
- Loose objects only, ask Git for packed ones: no packages, but packed objects are the common case in any repository
  that ran maintenance, so most commits would still start the processes this work removes.

Ranking: miniz_oxide readers > hand-written inflate > gix sub-crates > loose only.
The first beats the second because the decoder is maintained and tested upstream, at a cost of two packages.
The second beats the third because 34 packages is far larger than the readers' own surface and the readers are needed
anyway for the decline rules.
The third beats the fourth because it removes the starts the fourth keeps.

## Measurements

Binary size of the release executable: 8,707,536 bytes before, 8,887,064 bytes after, 179,528 bytes more (2.1 percent).

Release build time: the incremental release build of the crate took 24 seconds after the change.
The same step before the change was not timed on its own (the baseline run built three crates in 58.8 seconds),
so no before and after comparison of build time is claimed.

Latency, from `mise run //package/git-policy/cli:native:perf` (`bin/native-perf.ts`, 30 samples per scenario, paired with
direct Git in each run), run as alternating arms on a loaded host:
baseline binary A and new binary B, five runs of A and three completed runs of B (one B run stopped in the script's
own cleanup with `ENOTEMPTY` on the pack directory of its scratch repository, a race with the repository's
background maintenance that is not caused by the readers).

- Added latency of a forwarded `rev-parse`, medians per run: A 1.7, 1.7, 1.6, 1.9, 1.8 ms; B 0.9, 1.1, 0.9 ms.
  The spread inside the A arm (same binary, five runs) is 0.3 ms, and inside the B arm 0.2 ms,
  both smaller than the 0.8 ms difference between arms.
- Added latency of a forwarded `status`: A 1.7, 1.7, 1.8, 1.8 ms; B 0.9, 1.1, 0.9 ms.
- Added latency of an explicit-path commit: A 104.4, 102.8, 105.8, 104.6, 99.4 ms (range 6.4 ms); B 70.2, 73.1, 71.3 ms
  (range 2.9 ms). The commit is about 33 ms faster, with the arms far apart relative to the range inside each.

Git start counts per budget scenario (`src/native/binary_budget_tests.rs`, exact counts, strace of the process tree).
Before is the table at the start of this work, after is the table now; the configured-hook scenario's after value also
reflects the hooks removal that landed on main in the meantime.

- Plain commit: 38 Git starts before, 18 after; read-only starts 27 before, 7 after.
- Commit with one configured hook: 43 before, 18 after (the hook starts are gone on main); read-only 28 before, 7 after.
- Amending commit: 37 before, 17 after; read-only 27 before, 7 after.
- Second commit of the repository (warm): 40 before, 20 after; read-only 26 before, 6 after.

The remaining read-only starts are `cat-file` 3, `rev-parse` 2, `symbolic-ref` 1 (shadow) and `version` 1 (cold cache).

## Controls and what they did not exercise

Every reader has a differential control that runs real Git 2.56.0 and the reader over the same repositories and
compares standard output, standard error and exit code, and a control for every decline rule.
Covered: loose and packed objects with deltas, SHA-1 and SHA-256, annotated and lightweight tags, packed refs with
loose overrides, symbolic refs, linked worktrees (location read, Git paths and refs declined), a separate Git
directory, alternates, replace refs, shallow and graft files, reftable (declined), split index (declined),
index versions 2 to 4, intent-to-add, unmerged paths, gitlinks, file-to-directory changes, renames, merge commits,
non-UTF-8 and very long paths, symbolic-link start directories, include files and injected command entries.

Not exercised, and so not claimed: partial clones with promised objects (a missing object declines to Git, but no
fixture has a promisor remote), a real sparse index (declined through the `index.sparse` key and the `sdir` extension
without a fixture that has one), trees with millions of entries, an owner mismatch on the Git directory
(the check is in the reader and has no control because the controls cannot change a file's owner),
a device boundary on the walk up to `.git`, the index checksum (not verified; Git verifies it), a `/etc/gitconfig`
installed outside `/etc`, and Windows (the readers are Unix-only and every query starts Git there).

Guard controls were shown failing, one guard at a time on a restored tree:
disabling the discovery-key check, the file-to-directory check, the rename guard of the porcelain `diff`, the
unmerged and intent-to-add check, the ambiguous upstream check, the replace-ref and shallow check, and the switch each
made its matching control fail.
Disabling the reftable directory check alone did not fail a control, because reftable repositories are also declined by
their `extensions.refstorage` key; that check is a second guard on the same case.
