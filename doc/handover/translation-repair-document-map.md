# Translation repair document map

## Current status

From 2026-10-03 to 2026-10-05 a trial model,
MiMo 2.6 Pro,
worked the package on the branch `translation-repair-mimo-trial` while the owner was away.
Its start point and log is
[`translation-repair-mimo-trial-2026-10-03.md`](translation-repair-mimo-trial-2026-10-03.md),
whose "Outcome:
merged on 2026-10-05" records that the branch was merged by fast-forward and what was restored after it.

As of 2026-09-29,
work is on the LEGACY slice pipeline,
under the owner's direction decision of 2026-09-01 and the OpenRouter order of 2026-09-03,
with Synthetic,
Amazon Bedrock,
Hyper and OpenRouter as its providers.
It is not production ready.

Since 2026-09-27 the work has been the owner's audit of the whole package
("audit the whole translation-repair pkg for all the mistakes we've made and fix all of them").
The family's
[audit ledger](../../package/translation-repair/doc/audit-ledger.md)
holds every finding with its measurement,
its fix and its guard,
and every mistake made while fixing;
[`mistake-prevention.md`](../../package/translation-repair/doc/mistake-prevention.md)
holds the rule that prevents each family of mistakes.
Open work is every ledger entry whose status reads open or recurring.
The first pass of 2026-10-15,
`TianqiChen66624` at `db6c26a486`,
settled on 2026-10-15 and was read the same day
(the pass log's "TianqiChen66624 read,
2026-10-15"):
every check was clean,
the rebuilt page-level consistency pass carried one of its two repeats,
the two Gemma seats were withheld as before,
and her last message,
quoted in her own words,
shipped in the past tense,
as it had in `TianqiChen66623` (ledger M220).
The round that keeps quoted speech in its speaker's tense landed in `277145a862`,
and `TianqiChen66625` was launched on it the same day
(the handover's checkpoint of 2026-10-15).
It settled with every check clean and her last message in her own present tense;
its consistency pass carried the refrain to three of four places and left the other repeat,
and seven replies were refused for naming a shown label in another form;
the causes and fixes are in the handover's checkpoint of 2026-10-15.
The latest pass,
`TianqiChen66626` at `7c21a56fc2`,
settled on 2026-10-18 and was read the same day
(the pass log's "TianqiChen66626 read,
2026-10-18"):
no high-severity defect on its page,
the refrain and the repeat both carried,
and her last message in her own present tense;
no consistency run-off happened,
so the finalist extension was not exercised,
both Bedrock Gemma seats were dead all run on an expired token,
and the Synthetic meter was unreadable in every reading.
The owner's answers it serves are in the family's
[design commitments](../../package/translation-repair/doc/design-commitments.md)
under 2026-10-11.
The latest passes,
of 2026-10-22,
are filed in the
[answers-restart record](../planning/translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md):
`XingZ6018` ended with no page after every provider ran dry
(ledger B818 to B825),
`XingZ6019` ended on a short writing bench while NeuralWatt was dry
(ledger B826,
fixed on 2026-10-23 UTC),
`XingZ6020` was running when the record of `XingZ6019` was filed
and settled after 10 h 33 min over two attempts with the last 30 of 118 slices unfilled
(its read starts at
[its run file](../planning/translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md),
and its defects file lists the commit that fixed each finding),
and `XingZ6021` on the build `bcff23b8ce` settled in 2 h 5 min 45 s on 2026-10-25 (UTC)
with a page whose Shinonome section ships archive text that the source lacks;
the owner's rulings on the floors behind it are of 2026-10-25 and the change is in progress.
The read of `XingZ6021` starts at
[its run file](../planning/translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-run.md).

The finite redesign stopped after Candidate M failed on 2026-09-01.
No Candidate A through M is production-eligible,
and no successor implementation or calibration is authorized.
That closure does not bound the legacy pipeline.

Start with:

1.  [`translation-repair-handover-2026-09-06.md`](translation-repair-handover-2026-09-06.md),
    the current snapshot whatever its date:
    its "Where the work stands" opens with dated checkpoints,
    newest first,
    and its "What to do next" carries the state each session left and the next launch;
2.  the family's [audit ledger](../../package/translation-repair/doc/audit-ledger.md)
    and [`mistake-prevention.md`](../../package/translation-repair/doc/mistake-prevention.md)
    for the audit in progress,
    and the [family README](../../package/translation-repair/README.md)'s "Where the rest lives"
    for the family docs on configuration,
    the roster,
    the providers and the seats;
3.  [`translation-repair-readiness-signal.md`](../planning/translation-repair-readiness-signal.md)
    for what production ready would take and why the answer is still no;
4.  [`translation-repair-openrouter-2026-09-03.md`](../planning/translation-repair-openrouter-2026-09-03.md)
    for every OpenRouter run and its reading,
    one section per run or read,
    each heading dated;
    when this map was written the newest was TianqiChen66626's read of 2026-10-18;
5.  [`translation-repair-redesign-failure-2026-09-01.md`](../audit/translation-repair-redesign-failure-2026-09-01.md)
    for the redesign's terminal disposition and A through M reconstruction;
6.  [`translation-repair.md`](translation-repair.md)
    for the owner's operating rule,
    and for the private evidence paths,
    spent prompts and stop conditions of the 2026-09-01 handover it keeps;
    its repository state is that handover's,
    and the current one is the snapshot's "Repository state";
7.  this map for the remaining document families and its [redaction timing](#redaction-timing).

## Finite-interface design

- [`translation-repair-pipeline-redesign.md`](../planning/translation-repair-pipeline-redesign.md)
  records the original hard requirements.
- [`translation-repair-interface-comparison.md`](../planning/translation-repair-interface-comparison.md)
  is the candidate index and shared design baseline.
- [`translation-repair-interface-candidates-a-d.md`](../planning/translation-repair-interface-candidates-a-d.md)
  covers serial producers,
  specification compilation,
  brief-before-prose,
  and immutable-shell iterations.
- [`translation-repair-interface-candidates-e-g.md`](../planning/translation-repair-interface-candidates-e-g.md)
  covers conditional adoption,
  donor assembly,
  and realization ledgers.
- [`translation-repair-interface-candidates-h-j.md`](../planning/translation-repair-interface-candidates-h-j.md)
  covers bounded verdicts,
  candidate-scoped ballots,
  and the rejected Kimi expansion.
- [`translation-repair-interface-candidates-k-m.md`](../planning/translation-repair-interface-candidates-k-m.md)
  covers readable review units,
  lean realization,
  and risk-split challengers.

Candidate detail files are evidence,
not authorization to reuse their prompts or resume their graphs.
Before formatting,
the split script of 2026-09-01 assigned every byte of the pre-split comparison to the index or one detail file.
Its pre-split SHA-256 was
`933df3bccd51b986bd325d6fd540c4b972d4a5288ebbb205096cfba66398eac9`.
On 2026-10-13 the K to M file,
past the line limit,
became an index with one file per candidate,
its wording unchanged.

## Chronological history

[`translation-repair-history.md`](translation-repair-history.md)
is the chronological index.
It lists the groups;
each group's own index lists that group's files,
one subject or span of days to a file.
Every file opened directly says in its opening lines that it is historical evidence
and that the stop condition in its group's index applies;
each group index carries the Candidate M implementation stop as "Current stop condition".

The split of 2026-09-01 cut the pre-split source,
SHA-256 `56bdbdf2a0d5b7198b0ff75b428d7b00a2e9218ae28e25f9421895f5ce51aa8d`,
into 12 groups and,
by size,
44 segments named by number.
The
[`translation-repair-history-secondary-split-2026-09-01.json`](../audit/translation-repair-history-secondary-split-2026-09-01.json)
and
[`translation-repair-history-indented-split-2026-09-01.json`](../audit/translation-repair-history-indented-split-2026-09-01.json)
audits record exact raw-part reassembly for those segments,
and name files that no longer exist.
On 2026-10-13 each group's segments were joined again and filed by what they hold.
The titles,
stop banners and continuation headings the earlier split had added to each segment were removed,
one stop banner kept per group,
and headings were added where a long stretch had none.
The joined groups were compared word for word,
in order,
with the last committed pre-split file (`aef3c4cf55^`):
the only words that differ are the group titles,
the stop banners,
and the 2026-09-01 stop section,
which was written after that commit.

Historical claims can be superseded by later segments.
The timestamped failure report and current handover take precedence.

## Superseded handovers

The split of 2026-09-01 moved three dated snapshots out of the handover:

- 2026-08-29 pipeline and stopped-run state;
- 2026-08-25 to 2026-08-27 provider and production state;
- 2026-08-26 session close and open register.

The handover index also links the 2026-09-04 snapshot among these historical ones,
and the 2026-09-06 snapshot as current.

The pre-split handover had SHA-256
`285c54e35836750d110a544bd9e93ae08980b0cb989e4eef95820af478a670aa`.
Every pre-split byte was assigned to the current handover or one dated snapshot before formatting.
On 2026-10-13 the 2026-08-25 to 2026-08-27 snapshot,
past the line limit,
became an index with a file per subject,
its wording unchanged.

Dated snapshots are historical evidence.
Do not follow their next-action lists without reconciling them against the current implementation stop.

## Legacy production operation

- [`translation-repair-overlap-dial.md`](translation-repair-overlap-dial.md)
  records the overlap mechanism and measurements.
- [`translation-repair-run-continuity.md`](translation-repair-run-continuity.md)
  indexes supervision,
  restart measurement,
  and cache-fix history.
- [`translation-repair-corpus-pass.md`](../runbook/translation-repair-corpus-pass.md)
  is the manual corpus-pass runbook.
  Passes on the legacy pipeline are authorized;
  when the next one launches is in the current snapshot's "What to do next".

The pre-split continuity document had SHA-256
`14c7e46f7b54867af4539cca0ba493302da06ff5a7e16a76d32d8be18a281806`.
Every byte was assigned to the index or one of its three historical parts before formatting.

The final process-state audit is
[`translation-repair-process-state-2026-09-01.json`](../audit/translation-repair-process-state-2026-09-01.json).
It checked the exact pass and supervisor argument basenames recorded in the continuity documents and found no matching
process.

## Split verification

The sanitized after-state audit is
[`translation-repair-document-split-2026-09-01.json`](../audit/translation-repair-document-split-2026-09-01.json).

For the history,
handover,
run-continuity,
and interface-comparison families,
the audit verifies that every word from the pre-split or unreplaced committed baseline remains in order after formatting
and splitting.
It also verifies that every prior Markdown link target remains,
and that the current handover,
Candidate M disposition,
legacy stop banner,
and implementation stop are present.
The after-state probe is deliberately limited to ordered word tokens and link-target containment;
it does not claim punctuation,
Markdown syntax,
code syntax,
or link ordering.
Split-time byte-assignment and raw-part audits provide the separate structural evidence.

The final local-link audit is
[`translation-repair-document-links-2026-09-01.json`](../audit/translation-repair-document-links-2026-09-01.json).
It checked 168 local Markdown and JSON targets,
including 14 rendered heading fragments and one same-document fragment,
from translation-repair documents and inbound links from other documents;
none failed.
The audit parses Markdown with `remark-parse`,
derives visible heading text from its syntax tree,
and applies GitHub's heading slugger.
Its staged JSON retains one result row per checked target.
The
[`translation-repair-document-link-controls-2026-09-01.json`](../audit/translation-repair-document-link-controls-2026-09-01.json)
positive controls detect a missing file and a missing fragment while accepting a valid file plus fragment fixture.

## Provider and tooling evidence

[`../troubleshooting/README.md`](../troubleshooting/README.md)
indexes provider,
transport,
streaming,
token-limit,
concurrency,
and detached-worktree behavior.
Read the matching troubleshooting record before attributing an external-tool failure.

Candidate M's public harness and sanitized envelope evidence are pinned by SHA-256 in the failure report and current
handover.
The GitHub gist base URL is mutable;
use the revision-specific URL and byte digest.

## Task numbers in these documents

Many documents indexed here cite items of a session's task tracker,
as `#N` or as "task N".
Those ids belong to the session that wrote them:
one number names different work in different sessions,
none is a GitHub issue,
and a bare `#N` on GitHub links an unrelated one
(`AGENTS.md` rules XNS and XN2).
The history files,
dated snapshots,
run-continuity parts and interface-candidate files keep their wording and their numbers as written.
Where one of them passed the line limit it was filed into several files on 2026-10-13,
which moved text between files and added headings and nothing else.
The living documents name the work instead:
the translation-repair decision records,
the current handover and its snapshot,
the readiness signal and OpenRouter pass log it links as current,
and the translation-repair runbooks and troubleshooting docs.
Every other document keeps its numbers as written;
read each against the session and date around it.
Where a living document quotes the owner,
the quotation keeps its number and the text around it says what the number meant.
The sections those living documents carry from the sessions of 2026-09-10 to 2026-09-15
keep that tracker's numbers under a note saying whose they are.

## Redaction timing

The
[family redaction policy](../../package/translation-repair/doc/redaction-timing.md)
records the standing instruction:
documentation redaction is deferred while the package remains unfinished and not production-ready.
The split preserves historical corpus,
review,
calibration,
and takeover evidence for later package work.
Redaction becomes a required readiness gate before the package may be declared finished or production-ready.

The timing deferral does not authorize committing credentials,
API keys,
or raw provider payloads.
Those materials remain private throughout development and after release.
The targeted staged-content audit is
[`translation-repair-sensitive-material-2026-09-01.json`](../audit/translation-repair-sensitive-material-2026-09-01.json).
It checks credential and embedded-image signatures,
staged file extensions,
and exact long chunks from retained Candidate I,
K,
L,
and M provider-traffic artifacts without redacting corpus or reviewer wording.
