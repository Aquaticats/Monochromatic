# Audit ledger: the Markdown linter's gate and the split of long docs

The Markdown linter had no gate in the family and the findings an earlier fix cleared came back,
and the docs the owner's line limit then bound were split by subject or by a span of dates.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## D35: the Markdown linter had no gate in the family, and the findings D27 fixed came back

Status:
fixed 2026-10-22 (UTC),
found the same day when `mise run //:lint:monochromatic-lint` over the family's docs reported 90 findings on `main` at `05d6758249`,
all `markdown/semantic-line-breaks`,
in nine docs:
the audit ledger's file of folds and quote forms (10),
its file of follow-ups on emphasis,
definitions,
cache accounts and Synthetic quota (41),
its file of outcome rules and stage records (18),
the consistency pass file of the design commitments (5),
two files of this record's prevention rules (5 and 3),
the seats and calibration page (1),
the OpenRouter pass log's file of 2026-10-18 (4)
and the package split plan (3).
D27 had fixed every one of these docs with `--fix`.
The text added since was written in long lines,
and nothing failed.

Cause:
D27 recorded that the root `lint:markdown` gated nothing (tree-wide findings numbered in the tens of thousands)
and fixed the docs by hand without adding a task that reads the linter's output.
The family's tasks (`mise.toml` of each package under `package/translation-repair`) run oxlint,
the type check and the unit tests,
and no source-scan read a Markdown finding,
so a long line in a ledger entry was invisible to every gate.
The root task now named `lint:monochromatic-lint` walks the whole tree,
where 84,918 findings under `doc` alone (81,516 of them `semantic-line-breaks`) drown the family's.

Fix:

- `510de25a89` adds `markdown-lint-clean.unit.test.ts` to `source-scan` and to its `source-scans` task,
  which builds `adjudication` first for the keyless child fixture the test starts the linter through.
  It runs `monochromatic-lint` from the repository root over the package docs and READMEs,
  the living repository-level docs of `readLivingRepositoryDocs`,
  and `doc/planning/translation-repair-package-split.md`,
  which every package README links and no split audit pins
  (and,
  since the item on the older repository-level docs closed,
  the docs `OTHER_STEMS` names).
  It reads the linter's JSON lines and fails on any finding,
  each named `path:line:column rule`.
  A run that exits other than 0 or 1 (a missing path,
  a linter that cannot finish) fails the case and is never read as clean.
  `5247329ee3` moved the fixture case onto the findings helper,
  since the guard on scan comparisons rejects `toEqual` on a scan's findings.
- The fixture case runs the linter under the root configuration over a planted file that runs on past a comma
  (one finding at line 1,
  column 14),
  a planted file that breaks after it (none),
  and a path matching nothing (exit 2,
  refused).
  The docs case failed on the unfixed docs before any fix,
  with exactly the 90 findings in the nine docs this entry names,
  which shows it can find the class in the real docs.
- `881871abf7` and `932b4236b9` fixed eight of the nine docs,
  one line break to a finding.
  Each doc has the same words before and after and parses to the same tree of Markdown nodes
  (`~/temp/agent/tr-fan/w266-scratch/verify.ts`,
  which compares whitespace-normalised words and the parsed tree against `HEAD`);
  none holds a character outside the Basic Multilingual Plane,
  so `--fix` was safe (D27).
  The guard then reported only the ninth doc,
  `audit-ledger-outcome-rules-and-stage-records.md`,
  whose 18 findings another branch fixes.

Not covered:
the archives D27 keeps byte for byte
(the dated snapshots,
the history,
run-continuity and interface-candidate files)
are not read,
and the 2026-08-29 snapshot still carries its one finding.

The older repository-level docs
(closed 2026-10-22 (UTC),
the owner's veto open):
the first fix left unread the repository-level docs whose names do not start with `translation-repair`,
among them `doc/planning/the-third-rendering*.md`,
`doc/handover/two-lane-outcome-vocabulary*.md` and `doc/planning/wire-the-heading-aligner.md`,
which `doc-line-limit.unit.test.ts` counted as the family's through its own list of name stems.
D27 named none of them,
and the entry left it to the owner whether they join the living set.
The lead session decided that they do,
since a rewrap by the linter's `--fix` changes neither their words nor their parsed tree,
and the owner can veto the decision.

The rule that picks them:

- `OTHER_STEMS` in `living-docs.test-fixture.ts` lists name stems,
  and a Markdown file directly under `doc/audit`,
  `doc/decision`,
  `doc/handover`,
  `doc/planning`,
  `doc/runbook` or `doc/troubleshooting` is the family's when its name starts with one of them.
  A stem that is a whole doc name also takes the files that doc is split into.
  `doc-line-limit.unit.test.ts` and `markdown-lint-clean.unit.test.ts` both read the list,
  so the two guards cannot disagree about who the family's older docs are,
  and a new file under a listed stem is linted and counted the day it is added.
- A stem is listed when the doc it names is the family's by its name (the three stems the line limit already held)
  or by its history and its text:
  the doc was created under a family commit scope,
  its subject is the pipeline's lanes,
  stores,
  critics or probes,
  and it is a planning,
  decision or handover record of the family.
  A family doc whose name opens with no listed stem is not read until its stem is added.
- A stem that no doc's name starts with fails the guard,
  so a doc renamed away from its stem cannot leave the stem reading nothing.
- The guard failed on the unrewrapped docs first:
  5,510 distinct findings (5,511 linter records) in 34 of the 36 docs the list reads,
  all `markdown/semantic-line-breaks`.

What was done:

- The 34 docs went through `--fix`:
  1 audit doc (164 findings),
  6 decision records (115),
  8 handover files (1,953) and
  19 planning docs (3,279).
  The linter now reports nothing on all 36.
  None holds a character outside the Basic Multilingual Plane,
  so `--fix` was safe (D27).
- Each of the 36 has the same whitespace-normalised words and the same parsed tree of Markdown nodes as before
  (`~/temp/agent/tr-fan/w267-scratch/verify.ts`,
  which compares the file against `a334a5f794`:
  words with the `>` that opens a quoted line left out,
  since a rewrapped quote gains one per new line,
  and the tree with text values whitespace-normalised and code,
  inline code and HTML compared exactly;
  its controls show a split heading,
  a loosened list,
  a break inside a code span and
  a break after an ordered-list marker each read as different).

The rewrap then took `doc/planning/wire-the-heading-aligner.md` to 2,256 lines
(found when gate g151 ran this branch's head):
`doc-line-limit.unit.test.ts` held the doc to 1,509 lines,
the size it had,
under an allowance,
and the rewrap's added line breaks put it 747 lines past that
and past the 2,000-line ceiling of any allowance.
The doc was filed by subject,
as the owner's rule on over-long docs requires (M163 and M165),
never by a count.
The index keeps the path,
the pin,
the verdict
and a contents list that links each file with a summary.
Eight files hold the remaining sections whole and in their order,
each named `wire-the-heading-aligner-<subject>.md` and opening with a line that names the index:
the finding,
the options and the ranking (289 lines);
unwired modules and the size of the change (275);
prototype attempts one to five (362);
the derivability probe and the block-count gap (160);
the unpaired headings defect (259);
attempt six,
lexicographic scoring,
with its corrections and its prototype (398);
the ambiguous sections and attempt seven (302);
and the wiring attempt and the landing (164).
The index has 129 lines.
The allowance was removed,
since no file needs one,
and `ALLOWANCES` is empty.
The stem `wire-the-heading-aligner` in `OTHER_STEMS` takes all eight files,
so `doc-line-limit.unit.test.ts` counts them
and `markdown-lint-clean.unit.test.ts` lints them with no list change.
The text moved verbatim:
the index and the eight files,
less their added title lines,
the two lines that name the index and the contents list,
join with blank lines into the old file line for line,
and their 10,945 whitespace-normalised words equal the old file's at `0994fbf492`.
The cut turned seven pointers by position into pointers across files,
so each now names the file it means by a relative link:
"The ranking above" in the prototype attempts links the options file;
"withdrawn earlier in this document" in the attempt six file links the unpaired headings file;
"attempt TWO from earlier in this document",
"the correction above" and "recorded earlier in this document"
in the attempt seven file
link the prototype attempts,
the attempt six file and the options file;
"the earlier 60 of 172" in the landing file links the derivability file;
and "the verdict at the top of this document" in the unpaired headings file links the index.

Two more files past the bound
(found when gate g153 ran the head `506cdf4315`,
2026-10-22 (UTC)):
`doc-line-limit.unit.test.ts` failed its case "Markdown docs past the line limit"
on the NeuralWatt handover file (1017 lines)
and on the audit ledger's file of provider streams,
usage counts and retries (1010 lines,
after B807 moved in).
The handover failure was on `main` too:
handover edits had landed as docs-only commits,
and the length case runs only in the unit task of `source-scan`,
which no docs-only landing ran.
The prevention rule is in `mistake-prevention-docs-and-references.md`,
"The Markdown linter".

The ledger file was filed by subject in `0526994c14`:
B718,
B719,
B720 and B807 moved whole and in their order
into `audit-ledger-neuralwatt-replies-cut-at-the-default-effort-and-the-energy-estimate.md` (372 lines),
since all four are NeuralWatt calls cut at the provider's default reasoning effort
or estimated by their energy.
B804,
the stream guards,
stays with the streams.
The streams file is 646 lines.
The entries by id,
the contents and the one line pointer to B719 in the `zheermao22` run record follow the move.
This file took D35 from the file of doc conventions (D12 to D33)
so that the next entry here has room:
that file held 961 lines before the move and holds 754 now,
and this one 214 before this paragraph.

The handover file was filed by span in `42cfeaa6c9`.
It keeps its path,
the owner's terms and the record to the first run on NeuralWatt (271 lines),
and links three files,
oldest first,
each with a summary:
the rotation reads and the first fix rounds (303 lines),
the reboot,
the Gemma measurement and the katex shim (200),
and the landings,
the `zheermao22` restart and the state after K4 (310).
The last is the newest and holds the current state and the queue,
and the file that kept the path says so in its first section.
The text moved verbatim:
the 6,241 whitespace-normalised words of the old file equal those of the four files,
less the new titles,
summaries,
index lines and links.
Each new file says it is part of the handover snapshot and the snapshot lists it,
since `readLivingRepositoryDocs` takes a snapshot's files from lines that name the snapshot,
and the length scan takes every `doc/handover` file whose name starts with `translation-repair`.
No fixture changed.
The cut left no pointer by position that crosses files.
Fifteen links in seven planning records pointed at the old "Next" section,
and each now names the file that holds the text it cites.

The audit and troubleshooting docs
(closed 2026-10-22 (UTC),
the owner's veto open):
the stems of this entry reached no `doc/audit` or `doc/troubleshooting` doc whose name does not start with `translation-repair`,
and 1,432 findings stood in 20 audit docs and 171 in 14 troubleshooting docs.
The lead session decided that the family's own docs there join the guards,
and the owner can veto the decision.

- Guarded now,
  by one full-name stem each in `OTHER_STEMS` (`a8eb6c05b4`):
  the 20 `doc/audit` docs created under the commit scopes `module-translation-repair`,
  `module/translation-repair` and `translation-repair`
  (1,432 findings before,
  none after;
  the largest held 294,
  187 and 153),
  and six troubleshooting docs about the family's provider and audit runs
  (151 findings before,
  none after;
  the Synthetic docs,
  the pgrep wait loop,
  the barrel star export and the NUL bytes).
  No file in the six doc families starts with any of the 26 stems except its own doc.
  The guard failed first with 1,583 distinct findings and the line limit on the 1,130-line picture-reading audit.
- The rewrap went through `--fix` (`3d73cb074d` and `12c735b08d`).
  None of the 26 docs holds a character outside the Basic Multilingual Plane.
  Each has the same whitespace-normalised words
  (the `>` of a rewrapped quote left out)
  and the same parsed tree as before,
  by the D35 comparison against `HEAD`,
  whose controls read a split heading,
  a code span break,
  a loosened list and a marker break as different.
  One doc differs by what `markdown/link-image-style` asks for:
  the shortcut reference `[Synthetic]` of `synthetic-aggregate-concurrency-stall.md` became the collapsed `[Synthetic][]`,
  which renders alike and matches once the reference type and the empty brackets are left out.
- The picture-reading audit held 1,424 lines after the rewrap,
  so it was filed by subject (`4d6c99727e`):
  the index keeps the path and links five files of 189 to 440 lines named for what they hold.
  The index and the files,
  less their added title lines and contents list,
  join into the old text line for line,
  with nine pointers by position that now cross files turned into relative links.
- Left out.
  The three repository-policy docs (136 findings) govern the whole repository.
  Two troubleshooting docs belong to the oxlint plugin (4 findings),
  where the family is only the workload measured.
  Six hold only `rust/require-rustdoc` on fenced Rust (16 findings),
  each fence quoting mise,
  Oxc or repository source word for word,
  and the linter offers no exemption for a quoted fence,
  as the section "The Markdown linter" of `mistake-prevention-docs-and-references.md` records.
  Those six joined the guard on 2026-10-22 (UTC),
  after the repository owner decided to exempt the Rust fences of `doc/troubleshooting/**/*.md` from every `rust/*` rule
  (root `monochromatic-lint.config.jsonc`,
  block `troubleshooting-quoted-fences`);
  the two oxlint-plugin docs and the three repository-policy docs stay out.
