# Preventing mistakes: docs and references

Rules for the docs themselves:
current-state docs,
references in code and docs,
dates and clock times,
TSDoc parameter and return descriptions,
how long a doc may grow,
and the Markdown linter.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## Current-state docs

What happened:
the document map's current status still described the pipeline of 2026-09-06 three weeks later,
and said no new pass was authorized while passes kept running (D28);
the handover index put the kill-and-relaunch rule in a README section that had moved;
and the day after TianqiChen66621's page was read,
the current snapshot still listed reading it as a next step (M55).
The scan B102 added gave this doc a section of its own,
and "Copies of shared code" went on saying nothing fails on a shared module without its own test (ledger B103).
A batch's state file listed two agents as running after the session that launched them had ended,
their folders unwritten since 2026-10-06 (UTC) and nothing handed back,
and four days passed before a merge showed their patches missing (ledger M140).
The design commitments page went on calling the fixes of n166,
n168 and n169 "green and not landed"
after they landed with n171,
and had no record of the carry run-off of B641 (ledger M263).

The rule:
a page read,
an owner ruling built
or a change of plan updates the current snapshot's "What to do next" in the same sitting,
before the next piece of work,
and its newest checkpoint when the next steps change;
the map's "Current status" moves with them when what it states has changed.
A status paragraph carries the date it was written,
and points at a record by its name and date,
not by its place in a file that grows.
A state file's "running" line carries the check that proved it
(an agent's id answering,
or a folder written since);
after a session break,
each agent's folder times are checked before the line is trusted.
A change that adds a guard searches this doc for sentences calling the gap open
(`rg --line-number 'Nothing yet|is open' package/translation-repair/doc/mistake-prevention.md`,
and the guard's subject by name)
and rewrites each in the same change.
A landing does the same for the commitments page:
it searches every `package/translation-repair/doc/design-commitments*.md` file for the name of each round it lands
and for "not landed",
and rewrites each sentence that states the old state in the same sitting (ledger M263).

What enforces it:
habit,
and step 7 of the launch checklist;
no test can tell a current paragraph from a stale one.

## References in code and docs

What happened:
588 task-list numbers stood in the package's source,
tests,
docs and task descriptions as if they were references,
seven of them in lines the commands print (D22).
One number named different tasks in different sessions,
and on GitHub each named an unrelated issue,
so no reader could recover any of them.
Several sat beside claims the work they named had since overtaken.
The repository-level docs carried 1,206 more (D26),
and the current handover's open-work list was task numbers
naming two pieces of work recorded nowhere else.
Two cache-account paragraphs written for B24 said "same cache check as the paragraph before",
which names nothing once a paragraph is added between them (M60),
and a census found hundreds more references by position in the package and the living docs (D33),
five of them pointing the wrong way after the text around them moved.
A read-only agent's comments named a guard as "the check just made" (T8,
twenty-fourth batch),
and a search for that shape found five more code pointers by sequence in the tests
("the next check",
"the NEXT check down",
"the next line can show",
"the previous case",
"reaching the next line"),
which D33's scan does not read.
A trial fix's comment named its check as "the check below",
and the position scan turned the suite red before the change landed (M111).
Six case names and a comment cited "the fidelity cluster",
"the group cluster" and the like,
names a writer gave its own work that no ledger entry holds (ledger B333).

The rule:
a reference names something a later reader can open:
a ledger entry,
a doc heading or path,
a file or symbol,
a commit,
a date,
or the finding itself in words,
and never a position
("above",
"the paragraph before",
"the case before this",
"the next check",
"the check just made"),
which a later insertion silently repoints.
A group of work is cited by the name the ledger gives it
(an entry,
or a numbered batch of a section),
never by a name a writer made for it.
Open work goes in the ledger,
not only on a task list.
A queued item in the handover is written with its definition:
what is run,
on what,
and what it answers.
"Haiku cap reading" stood in the queue twice with no brief or note that defined it,
and was defined only when the lead looked for what it meant (ledger M316).
A GitHub issue is cited only after `gh issue view` shows it is the one meant,
and is added to the guard's list in the same commit.
Before a citation is rewritten,
the claim beside it is checked against the code,
since the work it named may have changed that claim,
and the number is read in the task record of the session that wrote it,
since the same number names different work in another.
An owner's words keep their number verbatim,
checked against the transcript or the file the owner wrote in,
and are listed in the guard.

What enforces it:
`package/translation-repair/source-scan/src/task-list-numbers.unit.test.ts`,
which fails on a sign followed by one to four digits
anywhere in the package's source,
tests,
docs,
README or `mise.toml`,
and in the living repository-level docs `package/translation-repair/source-scan/src/living-docs.test-fixture.ts`
locates,
unless it is a listed issue or sits inside a listed owner quotation,
and on a listed issue or quotation that no longer occurs;
it fails too on a number written after the word "task" (D31)
outside the listed takeover-era stretches,
which keep that tracker's numbers under their notes.
`package/translation-repair/source-scan/src/position-references.unit.test.ts` (D33) reads the same texts
and fails on "above" or "below" after any word that marks no comparison,
bound or placement,
unless what follows compares or names,
on a reference verb or a parenthesis around one,
on a sequence noun before "before this" or "after it",
on "earlier in this file" and its kin,
and on "the former" and "the latter",
outside its listed exemptions,
and on an exemption that no longer names a reference.
Its first version read positions only after a listed noun
and passed a couple of hundred after nouns the list lacked;
a guard built on a word list is measured against a broader scan before it closes an entry.
Nothing enforces the sequence form:
of the 33 lines in `src` matching "just" before a verb of reading or making,
or "the previous",
"preceding",
"next" or "following" before a line,
check,
statement,
block,
call,
case or loop,
the five fixed ones pointed at code;
the other 28 name a document's lines or blocks,
a parser's input
or a run's order,
which a word list cannot tell apart from a pointer.
A reviewer reads new comments and case names for them.

## Dates and clock times

What happened:
owner answers given between 00:00 and 04:00 UTC were dated by the local day before,
the voting rulings in 40 places (D12);
clock times were copied as `git log` and `find` print them,
in local EDT with no zone,
157 of them in 17 files,
and the scan that found them skipped 14 more written with seconds (D25).
It happened again on 2026-10-18:
the files' local times,
16:58 to 17:00 EDT,
were written as UTC in a TSDoc and a ledger entry,
four hours early,
and the zone scan passed them (ledger B672).

The rule:
a date or time is read from its source's own timestamp and written in UTC with its zone.
Transcripts and run logs print UTC;
`git log`,
`find` and `stat` print local time,
so they run under `TZ=UTC` with a local-format date.
An owner answer is dated by the UTC day of its transcript entry.
A range states its zone once,
after its second end;
a time whose zone cannot be recovered says so in words.
A time read from a local-zone tool and labelled UTC passes the zone scan,
so the offset is read from the tool's own output (`stat --format=%y` prints `-0400`)
or the tool runs under `TZ=UTC`,
`date --utc --reference=<file>` for a file.

What enforces it:
`package/translation-repair/source-scan/src/clock-time-zones.unit.test.ts` for clock times,
over the package's comments,
docs and README,
the decision records and the current handover;
dates by habit,
with the D12 census as the way to check them.
No scan can catch a local time labelled UTC:
the text holds the digits and the label and nothing says which tool printed them,
so a check of the words would pass or refuse every time alike.
The check that works is on the source,
the offset the tool printed.

## TSDoc parameter and return descriptions

What happened:
`AGENTS.md` rule TD6 says that `@param` and `@returns` descriptions take no articles,
and that they explain why and not what.
4,031 description lines of this package opened with an article,
because nothing read the rule (ledger B1000).
The lead measured how the repository practises it:
of about 22,900 such lines outside this package,
670 (2.9 percent) opened with an article.

The rule:
a `@returns` description and a `@param name -` description open with the thing described,
not with one of the six spellings of an article:
"Its child nodes",
"Whether it is a node".
An article inside a description is ordinary English and is left alone,
and a list whose first item lost its article loses the articles of its other items too,
unless dropping one changes the meaning,
as in a relative clause ("Winner the ballots give").
The other half of TD6,
that a description says why and not what,
is the writer's reading and the reviewer's.

What enforces it:
`tsdoc-leading-articles.unit.test.ts` reads every TSDoc block of the family's source and tests,
outside fences,
and fails on a description that opens with an article,
naming each as `file:line`.
Its green run proves the opening of each description and nothing more:
no test judges whether a description explains why,
so a passing scan is no proof of TD6 compliance.

## Docs that only grow

What happened:
the audit ledger,
the OpenRouter pass log and the handover snapshot were appended to at every finding,
run and checkpoint and never split,
until the ledger held 41,252 lines and 38 translation-repair docs together held 143,715 (ledger M163).
No reader could open one whole,
and an agent reading one spent its context on it.
The first split cut each doc at the heading before the limit and numbered the pieces,
so a file held whatever happened to fall inside its count and its name said nothing (ledger M165).

The rule:
a Markdown file holds at most 1,000 lines,
as the owner ruled on 2026-10-12;
up to 2,000 only where the lead records why 1,000 is too tight.
A doc past the limit keeps its path as an index:
what a reader needs first,
then a contents list saying what each file holds.
Each file beside it holds one thing a reader would look for,
a subject,
a phase of the work or a span of dates,
and is named `<stem>-<what it holds>.md`.
The name and the sentence under the file's title are true of everything in the file.
A name that needs a counter,
"other" or "continued" means the boundary is wrong:
move the boundary.
A dated log is filed by date span,
a reference doc by topic,
a record of phases by phase;
a ledger entry goes into the file of its family,
which is the family this page files it under.
A stretch with no headings is read and given headings at its real shifts before it is cut.
Files may be small;
none is padded or merged to even out sizes.
New text goes into the file whose subject it belongs to;
a file that nears the limit is divided by a narrower subject or a shorter span,
and the index lists the new files.
A reference definition stands in every file whose links use it.
A guard that reads a record reads its index and every file that says it is part of it.
`wc --lines` is read on every Markdown file a commit touches before the commit:
the split's handover edits took one file to 1,076 lines,
and the guard that reads the limit runs in `source-scan:test:unit` and not in the `source-scans` task (ledger M250).
The caps round placed two ledger entries in a file that then held 1,042 lines,
and `e8fb13c2a3` moved them to a file of their own (ledger M312).

What enforces it:
`doc-line-limit.unit.test.ts` reads the package's docs,
its README and the repository-level translation-repair docs,
fails on any past its bound,
fails on a file named by a counter or a bare number,
and fails on a file that names a record whose index does not link it;
`withFiles` in `living-docs.test-fixture.ts` gives each living-docs guard a record's index and the files that say they are part of it.
Whether a boundary makes sense is the writer's reading;
no test judges it.

## The Markdown linter

What happened:
ledger D27 passed the family's docs through the repository's Markdown linter once,
with `--fix`,
and wrote down that no task read the linter's findings.
No task was added.
Three weeks later 90 `semantic-line-breaks` findings stood in nine docs,
all in text written after the pass:
ledger entries,
pass-log sections and paragraphs of this record (ledger D35).
A one-time pass over a class of findings had gated nothing,
and the same shape recurred after the long-doc split (M163) and the zone-less times (D12):
the sweep was done by hand and the class came back with the next text.

The rule:
a doc written or edited in the family is linted before its commit:
`mise run //:lint:monochromatic-lint <the doc's path>`,
which reports nothing on a clean doc.
That root task does not apply the semantic-wrap configuration,
so a doc it passes can still fail the check that counts,
`mise run //package/translation-repair/source-scan:test:unit`,
whose `markdown-lint-clean` case applies that configuration.
A brief for an agent that writes docs names the source-scan check and not the root task:
round 8 reported the root task clean on the ledger of the plain reading with tables and the containers,
and gate g225 found two `semantic-line-breaks` findings at its lines 716 and 717
(fixed in `a45783f7d3`,
ledger M300).
A finding is fixed by breaking the line at a clause boundary,
after a comma,
a semicolon or a colon,
and nowhere inside a code span,
a link or an emphasis span.
`--fix` makes the same breaks,
but not on a doc holding a character outside the Basic Multilingual Plane (an emoji,
or a rare Han character),
where the linter's offsets go wrong (D27).
A fix of a class of findings across the docs ships with the guard that reads the class,
committed first and shown failing on an unfixed doc,
because a sweep that gates nothing is undone by the next paragraph.
A docs-only change that lands without a gate runs `mise run //package/translation-repair/source-scan:test:unit` first.
So does every other commit that edits Markdown,
a one-line note of the lead's included:
the lead committed a note in B965 before the task had run,
and it failed `semantic-line-breaks` at two lines (fixed in `d4fbf7d6e9`,
ledger M307).
The line-limit guard runs only in that task,
since the `source-scans` task lists the Markdown lint guard and not the line-limit guard,
so a handover or ledger edit can pass the linter and still cross 1,000 lines
(ledger D35:
gate g153 found two files past the bound that docs-only commits had left).
That task names no build,
so in a fresh worktree it runs after the install and the builds the `source-scans` task depends on
(document-structure,
markup,
run,
typography and adjudication);
on 2026-10-22 (UTC) a run without them failed every scan file at load with `ERR_MODULE_NOT_FOUND`
for the markup package's `dist`.
A merge of main into the branch runs that task on the merged tree before the push.
The scans that read docs (clock times,
position references and the Markdown lint scan) read the whole tree,
so a merge can bring in lines that neither parent's scan saw,
and a gate that scanned the tree before the merge cleared three such lines into main
(ledger M294:
the merge `ecbbae7fa3`,
repaired in `b18a4f7a40`).
The land script runs `//package/translation-repair/source-scan:test:unit` whenever it merges main
and refuses to push on a failure.
The limit of the check:
the scan imports the gate's `dist` builds,
so after a merge it reads the builds made before the merge,
and a merge that changes markup,
typography or document structure needs a fresh gate and not the scan alone.

What enforces it:
`markdown-lint-clean.unit.test.ts` in `source-scan` runs the linter over the package docs,
the READMEs,
the living repository-level docs,
the package split plan
and the older repository-level docs whose names open with a stem of `OTHER_STEMS` in `living-docs.test-fixture.ts`,
and fails on any finding;
the archives that D27 keeps byte for byte are not read.
A new family doc whose name opens with no listed stem is not read,
so its stem is added to that list when the doc is written
(`doc-line-limit.unit.test.ts` reads the same list),
and a stem no doc opens fails the guard.
The test finds the linter on the path that `mise` gives its tasks,
so it runs under `mise run` and fails with the error of a missing program elsewhere.
The linter is a `cargo:monochromatic-lint` tool the root `mise.toml` does not pin,
so a newer version may find more than the one that last passed.
The list now holds the family's `doc/audit` and `doc/troubleshooting` docs whose names do not start with
`translation-repair` (ledger D35),
each by its whole name.
Six of those names are troubleshooting docs that quote mise,
Oxc or repository Rust word for word:
`cli-git-temporary-worktree-policy-inputs`,
`mise-detached-worktree-trust`,
`mise-inline-node-arguments`,
`node-lookup-in-agent-command-context`,
`oxlint-type-aware-cgroup-memory`
and `forbidden-strings-builtin-rule-fixture-tokens`.
They joined the list on 2026-10-22 (UTC),
after the repository owner decided to exempt the Rust fences of
`doc/troubleshooting/**/*.md` from every `rust/*` rule.
The root `monochromatic-lint.config.jsonc` block `troubleshooting-quoted-fences` turns the rules off for the virtual
files of those fences (named like `mise-inline-node-arguments.md/1.rs`),
and the pattern reaches no other tree,
so every Markdown rule still judges those docs.

What the guard cannot read:
a family doc outside `doc/troubleshooting` that quotes Rust source in a fence cannot pass,
because the linter applies `rust/require-rustdoc` to fenced Rust there,
declares that rule unsuppressible
and has no directive that exempts a quoted fence;
only a configuration block like `troubleshooting-quoted-fences` exempts a path.
The quote cannot be altered to add documentation comments,
so such a doc is written so that its fences are the family's own code with documentation comments,
or it holds the quote as a plain text block.

## Names that were replaced

What happened:
the ledger's accounts of B141 and B160 named `everyStageHeard` as the reader of the cache,
the function was removed when the refine cache's two gates came to share one predicate,
and nothing in the ledger said so (ledger D36).
Comments of the same round cited a probe by "the parent session",
a source no later reader can open,
and kept a condition ("until a bench says otherwise") that no ruling states (ledger D37).

The rule:
a commit that removes or replaces an exported name searches the docs and the ledger for it.
A historical record keeps the name as written and gains an in-place note with the current name and the date
(the repository's rule for a moved identifier in a historical record);
a living doc takes the new name.
A comment that cites a measurement cites a file in the repository that holds its raw figures,
and a comment that states a condition cites the ruling that states it.

What enforces it:
habit and review.
