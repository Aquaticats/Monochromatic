# Translation repair session handover

## Trial handover to MiMo, from 2026-10-03

From 2026-10-03 the owner is away for about 48 hours,
and MiMo 2.6 Pro works the package on the branch `translation-repair-mimo-trial`,
cut from `translation-repair-rebased` at the docs commit that added
[`translation-repair-mimo-trial-2026-10-03.md`](translation-repair-mimo-trial-2026-10-03.md).
That file is the trial's start point and its log:
the state on 2026-10-03 with what was measured,
the owner's answers on authority and branch,
the queue
(fix B126,
the empty-label contributor link that fails one case on `98e94e584`;
a fresh whole-suite census;
T8 batches;
B123),
the commands verified that day,
and a "Trial log" each session appends to.
The snapshot named under "Current handover,
2026-09-06" stays the operating record;
its 2026-10-03 checkpoint points at the trial file.

## Current handover, 2026-09-06

The current session snapshot is
[`translation-repair-handover-2026-09-06.md`](translation-repair-handover-2026-09-06.md),
named for the day it was opened and kept current since.
Read it first:
"Where the work stands" opens with dated checkpoints,
newest first;
"Repository state" carries the worktree,
branch and corpus pin;
sections on the fifth to twenty-third defect classes follow,
the later classes being in the
[OpenRouter pass log](../planning/translation-repair-openrouter-2026-09-03.md);
and "What to do next" carries the state each session left and the next launch.
Since 2026-09-27 the work has been the owner's audit of the whole package,
recorded finding by finding in the family's
[audit ledger](../../package/translation-repair/doc/audit-ledger.md).
The 2026-09-04 snapshot,
[`translation-repair-handover-2026-09-04.md`](translation-repair-handover-2026-09-04.md),
still carries the four earlier classes,
the three pages that shipped,
and the seven steps by which a page is read.

The work in progress is the LEGACY slice pipeline under the owner's direction decision of 2026-09-01
and the OpenRouter order of 2026-09-03,
not the finite redesign.
Corpus runs,
model calls,
and implementation on that pipeline ARE authorized;
the redesign-scope prohibitions in "Superseded handover,
2026-09-01" remain in force,
and the readiness judgement lives in
[`translation-repair-readiness-signal.md`](../planning/translation-repair-readiness-signal.md).
The pipeline is not production ready.

ALWAYS KILL AND RELAUNCH is the owner's rule of 2026-09-06 for every pass:
when source changes while a pass is running,
kill the pass by pid,
build,
and relaunch the same entry into a fresh runs dir on the new build.
Never let a pass finish on a build a commit has superseded,
and never read its page as readiness evidence.
When the fix is known before the launch,
land it first and launch once.
The rule is written in the family's
[`package/translation-repair/doc/configuration.md`](../../package/translation-repair/doc/configuration.md)
("A source change while a pass is in flight means kill and relaunch")
and in the runbook's launch and restore steps;
it is deliberately not a root `AGENTS.md` rule.

## Superseded handover, 2026-09-01

### Start here

The finite redesign stopped after Candidate M failed its single pinned-Carena calibration.
Read
[`translation-repair-redesign-failure-2026-09-01.md`](../audit/translation-repair-redesign-failure-2026-09-01.md)
before changing code or spending another model call.
It reconstructs Candidates A through M,
separates evidence from hypotheses,
and records the implementation stop.

No Candidate A through M is production-eligible.
No Candidate M code entered production.
Do not implement Candidate N,
retry or continue a spent prompt,
change a spent prompt's route or ceiling,
repair a private response,
or add another producer,
reviewer,
resolver,
continuation,
or fallback stage.
On 2026-09-01 the owner delegated the direction decision,
and it is recorded in
[`translation-repair-post-redesign-direction.md`](../decision/translation-repair-post-redesign-direction.md):
the redesign stays closed,
but bounding the production consolidation machinery and a four-entry verification pass are authorized.
The redesign-scope prohibitions in this section remain in force.

### Repository state

- Worktree:
  `/var/home/user/worktrees/translation-repair`.
- Branch:
  `translation-repair-rebased`.
- Timestamped failure-report baseline commit:
  `2c5f1159d21563f519a5f7ea19f8d473a36bd419`.
- Candidate M preflight documentation commit:
  `8226daa11`.
- Timestamped failure-report commit:
  `2c5f1159d`.
- Unrelated dirty path:
  `.idea/.name`.
  Keep it excluded.
- The process registry returned zero running background processes.
  The exact `/proc` argument audit in
  [`translation-repair-process-state-2026-09-01.json`](../audit/translation-repair-process-state-2026-09-01.json)
  found no `corpus-pass.mjs`,
  `corpus-pass.ts`,
  `resume-run.sh`,
  or `resume-supervisor.ts` process.
  These are the pass and supervisor names recorded by the legacy continuity documentation;
  the evidence does not make a claim about arbitrary unrelated command names.
- Production still uses the pre-redesign slice pipeline.
- Production replacement by a finite whole-document architecture remains closed.
  The completion objective was redefined by the owner on 2026-09-01:
  it quantifies over one small,
  one medium,
  one large entry plus Carena,
  per the direction decision.

The finite-prototype branch is
`prototype/translation-repair-finite-pipelines`.
Its local and GitHub commit is
`d9305094fa9d02e4776c603c412b89ed14e1a890`.
Candidate M's foundation,
lifecycle,
GFP isolation,
and sanitized mutation summary are durable there.
Do not merge that branch into production.

### Candidate M terminal disposition

The exact public harness SHA-256 was
`6f0004e010d1477c3e2eea84287d03d21724495d6c2595d8fceef9131b3bb952`.
Its revision-specific GitHub artifact is
`https://gist.github.com/Aquaticat/6ff4fd2f600fc257064aa32eee4c9753/b6c9fb5ebf943a5cd8cfeebbb4c6a2fbbc1e6654`.
The run pinned prototype commit
`d9305094fa9d02e4776c603c412b89ed14e1a890`,
manifest digest
`96368249c30d54192120b9d45aaf6cc742e4b725442158d93172fed163d23aef`,
and corpus commit
`a80634a674f94861ea3b7056fba054ca9eab1a2c`.

All 14 static nodes terminated:

- eight provider exchanges;
- four completed nodes;
- four spent-unusable nodes;
- six deterministic skips.

Qwen produced the sole admitted candidate.
MiniMax's author stopped at 32,000 output tokens.
Qwen's publication-language challenge failed exact anchor admission.
Both GLM challenges stopped at 32,000 output tokens.
MiniMax returned clean nonself challenges in both roles,
but one family per role did not satisfy the two-family floor.
Qwen's clean fidelity challenge was self-clean and correctly did not qualify.

Selection recorded `evidenceFloorMet: false`,
`productionEligible: false`,
no admitted dissent,
and Qwen and GLM as abstaining verifier identities.
Independent restart replay made zero transport calls,
reproduced the result,
and left the pre-post-audit runtime tree byte-identical.
Complete source,
archive,
candidate,
and image reading independently rejected the page for fidelity,
structure,
actor-attribution,
and publication-language defect classes.

### Candidate M private evidence

Do not copy raw payloads,
credentials,
or image bytes from:

- `~/temp/agent/prototype-Carena-M-risk-challenger-20260901/`;
- `~/temp/agent/prototype-Carena-M-risk-challenger-20260901/calibration-post-audit-summary.json`;
- `~/temp/agent/prototype-Carena-M-risk-challenger-20260901/complete-page-review.json`;
- `~/temp/agent/candidate-m-restart-audit-20260901.json`;
- `~/temp/agent/audit-candidate-m-carena-20260901.mjs`;
- `~/temp/agent/audit-candidate-m-restart-20260901.mjs`.

Private post-audit summary SHA-256 is
`557b4b79f9cc6371658cfcb21b99237813ab41c7aa116808f648918b2d646b02`.
Private complete-page review SHA-256 is
`0e357d4a0aa39503411c3dd83a18449b8cca35c699e978c4206dbf90a79929be`.
Independent restart-summary SHA-256 is
`4d3c09f6f5346f98d1baf86d3c4b4ee0f4257a47aad0ab6c69963c4e0fcc1564`.
Output-root directories and files are mode `0700` and `0600`,
respectively.
The external summary and audit scripts are mode `0600`.

Keep credentials,
image bytes,
and raw requests and responses private.
Documentation wording redaction is deferred while the package remains unfinished and not production-ready.
The
[family redaction policy](../../package/translation-repair/doc/redaction-timing.md)
makes redaction a required readiness gate before a finished or production-ready declaration.

### Spent-prompt boundary

Candidates A through M are terminal under their tested contracts.
In particular:

- Candidate I's author and verifier prompts are spent;
- Candidate K's author prompts are spent;
- Candidate K's verifier prompts were not dispatched,
  but Candidate K is rejected and they are not authorized work;
- Candidate L's author prompts and three dispatched candidate-one verifier prompts are spent;
- Candidate L's candidate-zero verifier templates were skipped and are not authorized later dispatch;
- Candidate M's author prompts and six dispatched candidate-zero challenger prompts are spent;
- Candidate M's candidate-one challenger templates were skipped and are not authorized later dispatch.

Truncated,
malformed,
caller-guard-rejected,
deadline-aborted,
and indeterminate outputs remain spent evidence.
Do not reconstruct partial JSON or hidden reasoning.

### What remains authorized

The "Inside the redesign scope" and "Outside the redesign scope" paragraphs were written on 2026-09-01.
The "Inside" paragraph states the redesign scope's standing prohibitions,
which hold.
The "Outside" paragraph's account of legacy work is superseded by the 2026-09-04 snapshot:
the OpenRouter fallback landed on 2026-09-03,
and three entries have since been run and read.

Inside the redesign scope,
only documentation and evidence-preservation work remains authorized:
keep the timestamped report and this handover synchronized,
and preserve durable links,
digests,
commits,
and stop conditions.

Outside the redesign scope,
the owner's 2026-09-01 direction decision authorizes legacy-pipeline work:
bound the consolidation correction machinery so a normal run always terminates with a published page,
then run one fresh pass over the four-entry completion set and read its actual output.
See
[`translation-repair-post-redesign-direction.md`](../decision/translation-repair-post-redesign-direction.md).

### Reading order

1.  [`translation-repair-redesign-failure-2026-09-01.md`](../audit/translation-repair-redesign-failure-2026-09-01.md).
2.  This current section.
3.  [`translation-repair-document-map.md`](translation-repair-document-map.md).
4.  [`translation-repair-interface-comparison.md`](../planning/translation-repair-interface-comparison.md).
5.  [`translation-repair-pipeline-redesign.md`](../planning/translation-repair-pipeline-redesign.md).
6.  [`translation-repair-overlap-dial.md`](translation-repair-overlap-dial.md) and
    [`translation-repair-run-continuity.md`](translation-repair-run-continuity.md) only for the legacy pipeline.
7.  [`../troubleshooting/README.md`](../troubleshooting/README.md).

## Historical snapshots

- [2026-09-04 OpenRouter production session](translation-repair-handover-2026-09-04.md)
- [2026-08-29 pipeline and stopped-run snapshot](translation-repair-handover-2026-08-29.md)
- [2026-08-25 to 2026-08-27 provider and production snapshots](translation-repair-handover-2026-08-25-to-27.md)
- [2026-08-26 session close and open register](translation-repair-handover-2026-08-26-close.md)
- [Full chronological history index](translation-repair-history.md)
