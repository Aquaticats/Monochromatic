# Audit ledger: process mistakes, agents and briefs in the reference, edge and prose mask rounds

What went wrong in the rounds from the review of n149 to the review of n156 (n150 to n157),
and in the rounds that built the deferred findings of the review of n156,
reviewed them and added the quoted-tense rule (n159 to n161):
the rule breaches of each round,
as its author or reviewer disclosed them,
and two mistakes of the lead's own that ran through the first stretch of work.
The entries before these are filed in
[Audit ledger: process mistakes, agents and briefs in the consistency fix rounds](audit-ledger-process-agents-and-briefs-consistency-fix-rounds.md),
which stopped at M209 because it would otherwise pass the line limit.

The work of these rounds is filed in
[Audit ledger: the consistency pass, what the refusal run steps over](audit-ledger-consistency-pass-what-the-refusal-run-steps-over.md),
[Audit ledger: the consistency pass, span edges as whole units](audit-ledger-consistency-pass-span-edges-as-whole-units.md),
[Audit ledger: the consistency pass, elisions, quoted numbers and pairing](audit-ledger-consistency-pass-elisions-quoted-numbers-and-pairing.md)
and
[Audit ledger: the consistency pass, the prose mask and slices parsed alone](audit-ledger-consistency-pass-the-prose-mask-and-slices-parsed-alone.md),
and the work of n159 to n161 also in
[Audit ledger: the consistency pass, what the word test reads as shown](audit-ledger-consistency-pass-what-the-word-test-reads-as-shown.md),
[Audit ledger: the consistency pass, definitions inside containers, fragment landings and the mask memory](audit-ledger-consistency-pass-definitions-landings-and-the-mask-memory.md),
[Audit ledger: building the tense check](audit-ledger-tense-check.md)
and
[Audit ledger: labels and counts in messages](audit-ledger-labels-and-counts-in-messages.md).
M220,
the read that missed the quoted tense,
is filed in
[Audit ledger: process mistakes, claims and commit messages](audit-ledger-process-claims-and-commit-messages.md).
The entries after M223 are filed in
[Audit ledger: process mistakes, agents and briefs in the review, docs, archive and empty link rounds](audit-ledger-process-agents-and-briefs-review-docs-archive-and-empty-link-rounds.md)
(M225 and M227 to M233),
in
[Audit ledger: process mistakes, shell commands and tasks](audit-ledger-process-shell-commands-and-tasks.md)
(M224)
and in
[Audit ledger: process mistakes, claims and commit messages](audit-ledger-process-claims-and-commit-messages.md)
(M226).

Written on 2026-10-15 (UTC) from the accounts of the rounds,
each of which lists its own breaches
(the reviews `s217/review.md`,
`s221/review.md`,
`s224/review.md` and `s227/review.md`,
the hand-backs `s219/handback-draft-items.md`,
`s219/handback-n151-rest.md`,
`s222/handback-n154.md`,
`s225/handback-n156.md` and `s218/handback-n152.md`),
and,
for M221 to M223,
the hand-back of n159 (`s230/handback-n159.md`),
the review of n159 (`s231/review.md`)
and the list and hand-back of n161 (`s232/rules-broken.md` and `s232/handback-n161.md`),
and the lead's notes of 2026-10-14 and 2026-10-15.
The breaches of the docs rounds that wrote these entries are in their own hand-backs and are not filed here,
except those of the round that wrote M221 to M223 (n163),
which are M228.
The rounds are named as the neighbouring entries name them,
and a round name is not a task number.
The sizes of reads and outputs are the authors' own figures.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M210: breaches of the Read, output, pipe and edit-batch rules by the review of the reference round (n150)

Status:
happened on 2026-10-14 in the review that found B575 to B579,
B590 and B599;
every one disclosed by its author in the review (`s217/review.md`).

- Reads over 50 lines,
  about fifteen,
  of sources and outputs:
  `consistency-splice.ts` (100),
  `consistency-closing-marks.ts` (90 and 92),
  `restore-typography.ts` (152),
  `quote-neighbours.ts` (134),
  `nested-single-quotes.ts` (143),
  `consistency-run-steps.ts` (113),
  `consistency-paragraphs.ts` (124),
  `consistency-prose-text.ts` (70),
  `consistency-quote-references.ts` (52),
  `pass-consistency-decide.ts` (70),
  `pass-consistency-slices.ts` (70),
  `parse-slice-body.ts` (64,
  no limit given),
  and the probe outputs `v-typo.out` (100),
  `v-r2x.out` (60) and `v-typo2.out` (57).
- Terminal output over 50 lines twice:
  a search of the page option over `src/corpus-run` (moved on 2026-10-18 to
  `package/translation-repair/*/src/corpus-run`) (about 90 lines),
  and a search with context in `v-r2.out` (about 100).
- Pipes and `head`,
  three times:
  one search piped into `head`,
  and one command piped into a search twice.
- Probes written as `.ts` and run with `node`,
  and non-exported helpers and the test fixtures imported from `src/` and not from `dist`,
  as the harness copy of the author of n149 did.
- The replay was started with `systemd-run --setenv` and `--property=StandardOutput=file:`
  and not with the shell redirect that the brief gave
  (same command,
  same log path).
- A Write created `v-r2.ts` with raw U+2028 and U+2029 in a regex,
  because the writer expanded the escapes,
  and a one-line script repaired it.
- Five Edits to `review.md` in one batch,
  and two Reads in one batch more than once (independent files).
- The review read the lead's notes to see what the rulings said;
  they are not on the list of forbidden files.

Prevention:
the same as M189 and M209.
Sources,
hand-backs and logs are read in ranges of at most 50 lines,
a probe that needs a helper that `dist` does not export says so in its hand-back,
and the brief names the exact form of the replay.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".

## M211: breaches of the Read, output, build and edit-batch rules by the reference round (n151)

Status:
happened on 2026-10-14 in the work of B575 to B579,
B590,
B574 (its outcome) and B599;
every one disclosed by its author in the hand-back (`s219/handback-n151-rest.md`),
and by the lead's notes.

- Terminal output over 50 lines several times,
  and Reads over 50 lines,
  about fifteen.
- Two commands that read `dist` at once.
- Two Edits to one file in one batch,
  about seven times.
- An Edit with a read,
  a lint run or a build of the same file in one batch,
  about five times.
- Polling:
  `rg --count` and `pgrep` over a running suite.
- Standard error written to a separate file.
  The lead checked the separate file for `[FAIL]` and found none.
- The hand-back file held only a pointer to the returned message,
  so the lead copied the message into `s219/handback-n151-rest.md`.

Prevention:
the same as M189 and M206.
A hand-back is a file with the whole account in it,
and a unit's output is captured on both streams.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".

## M212: breaches of the Read, output, edit-batch and search rules by the docs round of the ledger (n152)

Status:
happened on 2026-10-14 in the work of B549 to B574 and M201 to M209;
every one disclosed by its author in the hand-back (`s218/handback-n152.md`).

- Terminal output over 50 lines once:
  a count over the doc directory and the sources (about 130 lines).
- Seven Reads over 50 lines:
  the renamed fixes file (51),
  the hand-back of n144 (57),
  the lead's notes (54),
  the hand-back of n149 (69),
  the index by id (70),
  the guards prevention file (56) and a list of risky spans (52).
- Two Edits to one file in one batch
  (the heading and the paragraph of B540),
  an Edit and a command that reads the same file in one batch,
  and a command that read a file written in the same batch.
- Searches that recurse under named subfolders of the work tree:
  the doc directory,
  the sources of `w158`,
  and the old name of the renamed file across the package and the repository doc directory
  (the brief asked for the last).
  The lead had counted the doc-directory form as a breach for n145,
  and the author left the ruling to the lead.
- A script of exact replacements rewrote the line breaks of three docs
  where hand Edits were meant.
  The lead accepted it.

Prevention:
the same as M202.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands".

## M213: breaches of the Read, output, pipe and edit-batch rules by the review of the edge round (n153)

Status:
happened on 2026-10-14 in the review that found B580 to B584,
B591 to B594 and B600 to B601;
every one disclosed by its author in the review (`s221/review.md`).

- Reads with a limit over 50,
  eleven times:
  `consistency-closing-marks.ts` (125),
  `consistency-edge-marks.ts` (110),
  `restore-typography.ts` (110 and 90),
  `quote-neighbours.ts` (100),
  `consistency-unread-units.ts` (100 and 86),
  `consistency-quote-bounds.ts` (80 and 56),
  `renders-as-nothing.ts` (75),
  `v-r2.ts` (66),
  `nested-single-quotes.ts` (60),
  `pass-consistency-decide.ts` (60) and a diff file (52).
- Terminal output over 50 lines five times:
  a listing of the files of `src` (about 190 lines),
  two searches with context in a test file,
  a search of `v-r2x.out` (about 55 lines)
  and a search of `p-typo.out` (102).
- One pipe:
  a command piped into a search to see why a probe failed.
- Two Edits to one file in one batch:
  `p-containers.mjs` (twice),
  `p-invisible.mjs`,
  `p-docs2.mjs` and `review.md`.
- An Edit and a unit run that read the same file in one batch (`p-invisible.mjs`;
  the Edit failed and the unit ran on the old file).
- `--setenv=TMPDIR=S/tmp` added to the prescribed `systemd-run` line for the probes that run the pass fixtures,
  and `.ts` probes that import test fixtures and non-exported modules from `src/`,
  where the brief names `dist`.
- A search whose alternation was given as a path,
  which returned an error but still printed matching lines,
  and an `rg` flag that does not exist.

Prevention:
the same as M210.
A probe that cannot run from `dist` is named in the brief with the fixture it needs.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".

## M214: breaches of the Read, output, batch, chain and polling rules by the edge round (n154)

Status:
happened on 2026-10-15 in the work of B580 to B584,
B591 to B594,
B600 and B601;
every one disclosed by its author in the hand-back (`s222/handback-n154.md`).

- Reads with a limit over 50,
  about twenty-five:
  `consistency-edge-marks.ts` (90 and 90),
  `consistency-fold.ts` (110,
  120 and 120),
  `consistency-count.ts` (70),
  `consistency-quote-references.ts` (60),
  `consistency-run-steps.unit.test.ts` (62),
  `consistency-edge-marks.unit.test.ts` (72),
  `consistency-paragraphs.ts` (94),
  `restore-typography.ts` (100,
  112,
  122 and 62),
  `nested-single-quotes.ts` (120 and 69),
  `quote-neighbours.ts` (142 and 97),
  `pass-consistency-shipped-marks.unit.test.ts` (97,
  52,
  56 and 60) and `consistency-quote-bounds.unit.test.ts` (110).
  The figures are from the author's notes and a few may be off by a line or two.
- Terminal output over 50 lines:
  the first baseline runs of three probes printed fixture logs without a redirect;
  several searches of the red logs (about 60 and 27 lines);
  and a search of a green log that printed 37.7 KB,
  after which a Read of its last 14 lines showed one line of 28 KB.
- Two commands that read `dist` at once,
  twice
  (the baseline typography batch and the baseline batch of the run and bounds probes).
- Two Edits to one file in one batch:
  `consistency-whole-end.unit.test.ts` (five) and `consistency-run-steps.ts` (twice,
  in two batches).
- An Edit and a read of the same file in one batch
  (`renders-as-nothing.ts` with a test run,
  and `p-invisible3.mjs`).
- A `;` in one shell call
  (three `diff` commands and a count).
- A search with backticks inside double quotes,
  which failed twice with no effect.
- The hand-back file got several Edits that were not appends,
  and the last two fixes were two Edits to it in one batch.
- Polling:
  lines of a running suite log were counted,
  and the output of a running unit was read once.
- `cp`,
  `mkdir`,
  `diff` and `sha256sum` were used in the shell,
  with no pipe and no loop.

Prevention:
the same as M189 and M210.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "How an agent waits".

## M215: breaches of the Read, output, flag and search rules by the review of the edge round (n155)

Status:
happened on 2026-10-15 in the review that found B585 to B589,
B595,
B596,
B602,
B603 and B606;
every one disclosed by its author in the review (`s224/review.md`).

- Reads with a limit over 50:
  `page-slice-rewrite.ts` (80),
  `consistency-whole-end.ts` (68),
  `consistency-run-steps.ts` (57),
  `quote-neighbours.ts` (70) and `p-w-edges3.out` (60).
- Terminal output over 50 lines four times:
  a listing of a probe output (about 130 lines),
  a search with context for an example (about 190),
  a listing of the ancestor `node_modules` (about 100)
  and a long listing of the probes of the review before (about 80).
- Short flags (`ls -la`),
  and a `sed --version` as a placeholder command,
  which is a `sed` call with no effect.
- The replay ran as a background shell command and not as a `systemd-run` unit.
- A search with `--glob` under `node_modules/.pnpm` of the work tree,
  a recursive search outside the scratch folder,
  which matched nothing,
  and a read of the modules of an ancestor `node_modules` by import in a probe.
- The lead's notes add an Edit with a read of one file;
  the review says that its probe files were edited one Edit per call.

Prevention:
the same as M210 and M189.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands".

## M216: breaches of the Read, output, batch, build and search rules by the prose mask round (n156)

Status:
happened on 2026-10-15 in the work of B585 to B589,
B596,
B602,
B603,
B606 and B607;
every one disclosed by its author in the hand-back (`s225/handback-n156.md`).

- Reads with a limit over 50:
  `restore-typography.ts` (100) and `consistency-fold.ts` (65).
- Terminal output over 50 lines five times:
  a listing of the files of a scratch folder,
  a deep listing of a data folder outside the work tree,
  a search of the probe copies for their output paths (about 100 lines),
  a probe run directly that printed 43 KB of fixture logs,
  and a search for word helpers over `src` (about 90 lines).
- Two Edits to one file in one batch:
  a probe,
  `typography-prose-mask.ts`,
  its unit test,
  `translate-slice-key.unit.test.ts` and `restore-typography.unit.test.ts`.
- An Edit and a read of that file in one batch:
  `typography-prose-mask.ts` with the lint run (the log was discarded),
  `restore-ellipsis.ts` with the build (rerun),
  and `consistency-shown-edge.ts` and `nested-single-quotes.ts` with a Read,
  three times.
- Two commands that read `dist` at once:
  the two counters,
  run as two units in one batch.
- A `;` in one shell call (a count and an `echo`).
- The task `cache-account-audit` depends on `build`,
  so it rebuilt `dist` from the source of that moment
  (it ran no probe and no test).
- A metadata `find` over the runs directory of the replay,
  for slice-cache files newer than the setting of the cache version and a control since January.
  No file was opened,
  but the brief says never to open files there,
  and the control found nothing,
  so the check proves nothing (B607).
- Backups of the guard demonstration were restored with `cp` into the source,
  inside the write boundary and checked by checksum.

Prevention:
the same as M189,
M210 and M214.
A task that builds is named as a build in the brief,
and a check that cannot show a positive control is not offered as evidence.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Tasks,
builds and bulk output".

## M217: breaches of the output, Read, batch and foreground rules by the review of the prose mask round (n157)

Status:
happened on 2026-10-15 in the review that found B597,
B598,
B604,
B605 and B608 to B615;
every one disclosed by its author in the review (`s227/review.md`).

- Terminal output over 50 lines three times:
  `git status --short` in the worktree (about 290 lines),
  the foreground run of a probe that printed its fixture logs (about 130),
  and a numbered listing of a test fixture with a count limit (70 lines).
- Reads with a limit over 50:
  the lead's notes (53),
  `quote-style-unify.ts` (60,
  twice),
  `consistency-shown-edge.ts` (71) and `translate-guillemets.ts` (85),
  and Reads of small output files with no limit given.
- Two Edits to one file in one batch (`p-opening-marks.ts`).
- A Write and a command that read the same file in one batch (`p-fragments.ts`).
- A run of more than a minute started in the foreground
  (`p-long-paragraph.ts`):
  the harness moved it to the background after 170 s,
  the review stopped it with the task-stop tool,
  and a bounded version ran as a `systemd-run` unit.
- `git show HEAD:<file>` redirected into the scratch folder,
  a git read with no git write.
- Probes that were not over a minute were started as units,
  which the review called harmless over-use of the unit form.

Prevention:
the same as M210 and M214.
A probe that may run over a minute is started as a unit with a bound from the first run.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "How an agent waits".

## M218: ten fix rounds went by without a commit, each round's work left uncommitted in one worktree

Status:
happened from 2026-10-14 to 2026-10-15 in the lead's own handling of the rounds
(the lead's count in its notes:
"nothing has landed in ten rounds");
found by the lead while deciding to land,
and recorded by the lead.

Every round's tests and code stayed uncommitted in `w158`,
and the next round started on top of them.
The worktree held more than 230 changed entries by the report of n154 and about 290 by that of n157,
among them `mise.lock` and a handover document that n154 did not touch.
The cost showed in the reviews:
no build from before a change was kept,
so the reviews could not say whether a defect predates a round
(W3 of B602,
the shapes of B605,
the old scanner of B606 rebuilt from `HEAD`,
which is far behind the worktree and could not be proven byte-identical to the file as it was before n156),
baselines that were copies of the output of a build taken before the round and not a commit,
and guard experiments that restored files from backups in the scratch folder,
checked by checksum.
The rule `GCE` of the repository says that a work step is committed before the next one,
and that broken states are committed too.

Prevention:
each round's tests and code are committed before the next round starts,
even when a later round will change them,
and the brief of the next round names the commit it starts from.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M219: the review loop was not stopped while four builds gave identical results

Status:
happened from 2026-10-14 to 2026-10-15 in the lead's decisions to launch another review after each fix round;
found by the lead on 2026-10-15,
after advice,
and recorded by the lead.

Four builds (n149,
n151,
n154 and n156) gave identical oracle grids (the runs `s216`,
`s220`,
`s223` and `s226`),
identical run sites (50),
identical replay rows and identical recounts of the elisions,
and the reviews of those builds still found new defects each time,
in code next to the code the round had changed
(seven in the review of n149,
eight in that of n151,
nine in that of n154 and eleven in that of n156).
The checks that measure what ships had said the same four times,
and the reach measured for the findings was zero pages,
or exposure at a few dozen places.
The lead judged that another round of fixes would be followed by another review of the same kind.
The lead stopped on 2026-10-15:
land now,
record the findings of the last review as decided and not yet landed (B597,
B598,
B604,
B605,
B609 to B615),
and build them after the launch as their own commits.
They were built as n159 and landed on 2026-10-15 together with the fixes of n161,
in the tests `5fe3173cc5` and the code `277145a862`,
and not as their own commits.

Prevention:
when the checks that measure what ships stay identical across builds
(oracle grids,
corpus counts,
replay)
and each review finds defects in adjacent code,
the corpus reach of what is open is measured,
the work is landed,
and the rest is fixed after the launch.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M221: breaches of the Read, output, batch and hand-back rules by the round that built the deferred findings (n159)

Status:
happened on 2026-10-15 in the work of B597,
B598,
B604,
B605 and B609 to B615;
every one disclosed by its author in the hand-back (`s230/handback-n159.md`),
and one more found by the review of the round (the paragraph that begins "Found by the review of the round").

- Reads with a limit over 50:
  `consistency-paragraphs.ts` (94),
  `consistency-closing-marks.ts` (70),
  `consistency-count.ts` (80),
  `quote-style-unify.ts` (120),
  `consistency-prose-text.ts` (90 and 75),
  `refine-slice-key.unit.test.ts` (75)
  and `page-definitions.ts` (56).
- Reads with no limit,
  or over 50 lines:
  `claims-n132.mjs` (144 lines),
  `lib2.mjs`,
  and the copy of `grid-n132.mjs` (read before a context compaction,
  its contents too large to retain).
- Terminal output over 50 lines not sent to a file:
  an `rg --files-with-matches` for `module-logger`,
  twice (about 350 lines),
  an `rg` of the `mise.toml` files (about 280),
  a probe run through `mise exec` (about 140),
  an `rg` over the tests for a `describe` opener (persisted by the tool to a results file),
  an `rg` of the lost `**cats**` texts (about 250),
  and an `rg` over the whole `src` tree for guard anchors (122 KB,
  persisted by the tool).
- The structure of the hand-back:
  sections 1 and 2 went in one Write,
  sections 3 to 6 were appended with several Edits in places,
  and earlier text was edited in place several times,
  where the brief said one Write and then one Edit per appended section.
- Two or more Edits to one file in one batch:
  `typography-link-text.ts`,
  `consolidate-key.unit.test.ts`,
  `consistency-fold.unit.test.ts`,
  `consistency-shown-edge.unit.test.ts`,
  `quote-style-unify.ts`,
  `restore-typography.ts`,
  `restore-ellipsis.ts`,
  `consistency-prose-text.ts`,
  `consistency-paragraphs.ts`,
  `consistency-run-steps.ts`,
  `consistency-closing-marks.ts`,
  `consistency-count.ts`,
  `translate-guillemets.ts`,
  `quote-normalize.ts`,
  `consistency-shown-edge.ts`
  and the hand-back itself.
- A parallel `rg` of a log in the same batch as the run that wrote it (an ordering slip with no effect).
- `mv` to rename `typography-prose-mask-links.unit.test.ts` to `typography-link-text.unit.test.ts`,
  and `cp` to copy probes (inside the scratch folder and from read-only inputs).
- Waits written as `systemd-run` units:
  a placeholder run of `/usr/bin/true` and two runs of `/usr/bin/sleep`,
  used as foreground waits and not as polling.
  A `diff` of two text files,
  and a probe run with the path of the build from before the round (read-only).

Found by the review of the round and not by its author:
the hand-back called the folder `S/grid2` the "`s226` harness copies",
but the folder `s226` holds outputs only and no harness,
so the description is unsupported,
and the author's control reproduces the `s226` record only on the columns
that do not depend on the oracle or on the final-page check (B597).
The author had said it did not read the harness.

Prevention:
the same as M189,
M210 and M214.
A hand-back is one Write and one Edit per appended section,
and a Read states its range.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".

## M222: breaches of the output, Read, batch and hand-back rules by the review of the deferred findings round (n160)

Status:
happened on 2026-10-15 in the review that found B616 to B618,
B620 to B622 and B624 to B626;
every one disclosed by its author in the review (`s231/review.md`).

- Terminal output that could pass 50 lines:
  an `ls` of the two `dist/final/node` directories (several hundred lines),
  an `ls` of the probe folder of the previous round (about 66 lines),
  and the console output of the first run of `p2-pages.ts` (54 KB,
  kept by the tool;
  redirects were added afterwards).
- Reads with a limit over 50:
  `consistency-unread-units.ts` (56),
  `typography-shown-view.ts` (57)
  and `typography-link-text.ts` (52),
  and Reads of several small files with no limit given
  (each under 50 lines:
  `c-br-lines.txt`,
  `c-br-xing.txt`,
  `d-qnorm.diff`,
  `c-addrlabel.txt`,
  the outputs of probes `p9` and `p11` to `p16`,
  and a few more).
- A `cp --target-directory` of 14 probes of the author into the review's folder
  (inside the scratch folder,
  which the brief does not forbid;
  the hand-back of n159 counted the same as a breach),
  and an `rg` of the copies in the same batch as the `cp`.
- Fourteen Edits of the review file beyond the one Write and one Edit per appended section:
  four small corrections before the advisor check and ten afterwards.
- Two probes edited after their first run to take the module under test from the arguments
  (to run the build before n159 as a control);
  nothing else read those files in the same batch.
- The replay started with the tool's background flag and not as a `systemd-run` unit,
  since the brief's command was not a `systemd-run` line;
  every later run over a minute used the unit form,
  one unit at a time.
- A double-quoted `rg` pattern with backticks that the shell interpreted,
  which failed with no effect on any file.

Prevention:
the same as M210 and M214.
A search of a folder that may be large names a count limit or a file,
and a Read states its range.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Tasks,
builds and bulk output".

## M223: breaches of the Read, output, batch and build rules by the quoted tense round (n161)

Status:
happened on 2026-10-15 in the work of B616 to B618,
B621,
B622 and B625 to B628;
every one disclosed by its author
(`s232/rules-broken.md` and the hand-back `s232/handback-n161.md`).

- Reads with a limit over 50,
  of sources read while tracing the consistency pass:
  `artifact-two-lane-consistency.ts` (90),
  `consistency-count-types.ts` (70),
  `pass-consistency-decide.ts` (60 and 110),
  `pass-consistency-apply.ts` (120),
  `pass-consistency-apply-record.ts` (110),
  `pass-consistency-settle.ts` (145),
  `consistency-markup-ranges.ts` (92),
  `typography-prose-mask.ts` (60),
  the test file of B616 (55),
  `editor-calibrate-run.unit.test.ts` (56)
  and `files-newer-than-start.txt` (56).
- Whole-file Reads with no limit after a context compaction:
  `audit-sensitivity-print.unit.test.ts`,
  `rot-audit.ts`,
  `rot-audit.out`,
  `unit-pre-fails.txt` and `results-rerun.txt`.
- Terminal output over 50 lines not sent to a file:
  the listing of the files of `corpus-run` that start with `pass-consistency` (about 100 lines),
  the listing of the source files that mention tense (about 70),
  the search of the package `mise.toml` for its tasks (about 200),
  an unfiltered search for NUL names (about 45),
  a search of fragment and test names (about 100),
  a search for `shuffle`,
  `seeded` and `rotat` (about 70),
  a grep of the tests for `Number.MAX` (about 50),
  an `rg --line-number` for `messageNamesOnly` over `src` (about 140,
  where a per-file cap was meant,
  which does not cap the total),
  and a `git show HEAD:` of `typography-prose-mask.ts` with `--no-patch`,
  meant as a no-op,
  which printed the whole file (about 300).
- Two or more Edits to one file in one batch:
  the edits of B621 and B622 to `typography-prose-mask.ts`,
  and the edits of `audit-sensitivity-print`,
  `producer-calibrate-round`,
  `roster-bench-round`,
  `probe-verify-collect`,
  `judge-fidelity-probe-run`,
  `english-usage-policy` and `consistency-markup-ranges`.
- A source edit in the same batch as a build of the package
  (an edit to a test file,
  which the build does not read).
- Two Edits of files created before the compaction were refused as not read and redone after a Read,
  which breaks no rule.
- The probes and the lint tasks ran as `cd -- <path> && mise exec -- node <probe>` and `... mise run ...`,
  as the commands of the brief do,
  and several `diff` commands ended with status 1,
  which is no error.

Prevention:
the same as M210,
M214 and M216.
After a context compaction a file is read in ranges of at most 50 lines,
as before it,
and a search that may print more than 50 lines goes to a file.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Tasks,
builds and bulk output".
