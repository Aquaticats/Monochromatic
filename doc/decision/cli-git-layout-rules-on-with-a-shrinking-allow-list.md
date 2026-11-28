# The git wrapper keeps the layout rules from now on, with today's loops on a list that only shrinks

Status:
accepted by the owner on 2026-11-25,
answering item 16 of `package/git-policy/cli/doc/debt-ledger-2026-11-25.md`.

## Decision

The layout rules `AGENTS.md` sets for every crate
(LYD,
a `uses/` folder of links in every source folder;
LYL,
no two folders using each other;
LYF,
no loop among a folder's files)
are switched on for `package/git-policy/cli` now,
through the repository linter's rules that enforce them
(`rust/folder-uses-links` and `rust/file-order-in-folder`,
which `monochromatic-lint.config.jsonc` turns on only for the ZFS binding package today).
The loops and mutual uses that exist today go on an allow-list that may only shrink,
so new code cannot add one,
and later rounds untangle the crate folder by folder.

The owner chose "Enable now,
untangle over time (Recommended)"
over untangling everything before other work lands,
which would pause the ZFS route,
and over recording the crate as an exception.

## Why

The ledger measured,
on 2026-11-25:
29 of the crate's 55 source folders in one strongly connected set,
97 pairs of folders that use each other,
and a `uses/` folder in 19 of the 55.
The base folder `foundation` uses six higher folders
and is used by `landing` in 115 files and `worktree` in 142.
Switching the rules on with a shrinking list stops the growth at once,
keeps the ZFS route (the owner's first priority) moving,
and ends with the rules holding.

## What it costs

An allow-list of today's pairs to keep until it is empty,
`uses/` folders for the 36 folders that lack one,
and the untangling rounds after it.

## When the rules land (owner, 2026-11-25T22:40-05:00)

The round that built the lists (branch `chore/git-policy-cli-layout-rules-on-with-a-shrinking-allow-list`)
matched `main` at 17:42;
by 22:35 the merges since then left 63 list lines that tolerate nothing
and about 190 findings no line records,
so the lists are regenerated against `main` before they land.
Asked whether that happens before or after the crate split
(`doc/planning/cli-git-source-layout.md`,
"Splitting into crates (owner,
2026-11-04)"),
the owner chose "Before the split (Recommended)":
the lists are regenerated once the open wrapper rounds merge,
the branch merges,
and then the reformat and the split run.
Each crate the split creates is switched on with an empty list,
and every split round deletes the list lines of the folders it moves.

The linter that carries the `allow` option this list needs is `monochromatic-lint` `0.5.2-alpha.9`,
locked as `ef648a619e`,
with the option's own branch merged as `3cefd4f7c5`.
The alpha.9 release pass
(`~/temp/agent/monochromatic-lint-alpha-9-release-review-2026-11-25/keep/review.md`).
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
It found three defects,
and the round that fixed them is on `main`,
released as `alpha.10` (`6cfbcbb5f4`,
whose note names them):
a `no-file-list` line switched off every other file check for its folder,
so a new file added to such a folder and not listed there passed;
finding keys were not unique,
so one line tolerated every finding that shares its key;
and a folder in both a folder loop and a group loop could make a list that no set of lines satisfied.
The gate also scanned an unclosed `[` once per bracket.
`toolchain.lock.json` still locks `0.5.2-alpha.9`,
so the lock's bump to `alpha.10` is a step of its own.

The `uses/` links for the 36 folders that lack one,
named as a cost in "What it costs",
cannot be written while their folders loop:
the round measured `cargo package --list` giving no answer after 120 seconds with the 421 links in place,
against 1.9 seconds without them
(`doc/planning/cli-git-source-layout.md`,
the section the round's commit `457a1f6142` added),
so each folder's links are written when its uses no longer loop.

## What the list holds (owner, 2026-11-25T22:45-05:00)

The round put on the list every finding of the two rules on the day they went on,
not only the loops and mutual uses this decision named:
at its count 421 `uses/` links not yet written,
59 folder docs without a file list,
49 `pub use folder::*` re-exports,
20 findings on the `health/` group folder
and one function beside `fn main` in `main.rs`,
beside the 524 folder uses and 544 file uses inside today's loops.
Asked whether to keep that or list only the loops and the links,
the owner answered "Everything now,
untangle over time.":
every finding of the two rules on the day the rules land goes on the list,
and each line is deleted when its finding goes.

## Wiring uses merge with their findings unrecorded (owner, by 2026-11-28T01:54:18-05:00)

The linter holds "may only shrink" this way
(`package/linter/monochromatic-lint/src/layout_folder_links_tolerated_uses.rs`,
`use_joining_a_tolerated_loop`):
a new use of one folder by another that joins a loop the list already tolerates is a finding with an empty key,
so no line of the list can record it.
Most of this crate is one such loop,
so the round that wires a built-ahead piece of the ZFS route into the command that reaches it adds exactly such a use.
On the night of 2026-11-27 two rounds met it.
The round that admits `cancel-worktree-removal` and `settle-worktree-clone-record` into the management grammar
wired both commands,
and its branch prints 11 `rust/folder-uses-links` findings no line can record.
The round that built the wrapper's side of the lost-event triggers left its check before a command with no call site,
so that its branch stayed clean.

Asked after 2026-11-28T01:51:02-05:00 how wiring uses get through,
the owner was shown these ways,
in the order the coordinating session ranked them:

- record them:
  the linter gives such a finding a key and a wiring round adds its line,
  which amends "may only shrink" and needs a linter release;
- restructure each wiring so that it adds no new folder use,
  which puts the glue where a use is already recorded and not where it belongs;
- merge the wiring with its findings unrecorded,
  and untangle later;
- untangle first,
  which makes the route's wiring wait.

The owner answered "Merge red,
untangle later".

What follows from the answer:

- A round that wires a built piece of the ZFS route into a command merges with the `rust/folder-uses-links` findings
  its wiring adds.
- No line is added to `package/git-policy/cli/layout-allow-list/folder-rule.txt` for them:
  the list still only shrinks.
- The package's lint is red on `main` by those findings until the untangling removes them.
  `doc/handover/cli-git-rust-implementation.md` keeps the list of every such finding `main` prints,
  with its file,
  its line,
  the folder that uses and the folder used,
  and a wrapper round's brief names that list,
  so that a finding outside it is read as a new tangle.
- The untangling is the crate split's work
  (`doc/planning/cli-git-source-layout.md`).

The coordinating session's reading,
with the veto open:
the answer covers the wiring of a built piece into the command that reaches it,
and any other new use inside a tolerated loop is still a finding to remove before its branch merges.
