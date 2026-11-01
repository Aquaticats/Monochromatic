# Translation repair: the state before the reboot of 2026-11-01

On 2026-11-01 (UTC) the owner wrote:
"When you get to just before the launch I want you to not actually launch and prepare for a reboot and tell me when
it's a good time to reboot."
This file records the state at that point,
so the work resumes from the repository alone if the lead's scratch folder does not survive the reboot.
The snapshot named in [`translation-repair.md`](translation-repair.md) under "Current handover,
2026-09-06" stays the operating record for everything older.

## The branch

`translation-repair-rebased` is at `2bc1ec89533ba508d5c847c3276d7af3c6855c2e`,
landed on 2026-11-01 as a fast-forward over `e9b966af55`.
It carries rounds w406 to w409d of the owner's audit,
ledger entries B1345 to B1409 in the family's
[audit ledger](../../package/translation-repair/doc/audit-ledger.md).
In short:
the router refuses every ask of a held model on its held provider for a seat whose role honours the hold,
first asks included;
every production request names its seat,
and a request that names none is held;
a seat the recovery round loses as unreachable leaves the quorum denominator;
an explicitly empty refiner roster keeps the slice's incumbent text;
and the Markdown width scan reads fences per CommonMark and counts display columns.

The evidence for the landing:

- Gate g409 at that commit ran every family package's build,
  `lint:types`,
  oxlint and `test:unit`,
  and the scans:
  every step exited 0,
  with 3416 tests passing and none failing.
- The rotation sweep s409 at that commit was clean:
  no case failed under a salt that salt 0 passes,
  and none was red with no shift.
- Two second passes read the rounds' commits (same vendor as the authors, so not independent).
  The code reader found no major finding.
  The docs reader found three major findings,
  all in docs and none changing how a run behaves;
  they went to round w410.
- Corrective commit comments for 31 commit messages were posted after the push,
  `commitcomment-207662429` to `commitcomment-207662502`.

## The frozen build and the launch that did not happen

The frozen build was rebuilt in the branch worktree at `2bc1ec8953`
with the same tool links the gate and sweep trees used.
Its digest is
`sha256-tree-v1:5722d6027eb68ee39bf614ea25ee0b43437a3752827d49283cdb5b9b39c0946f` (3 files),
and the lead's launcher holds it as the frozen build.

The keyed pre-launch plan at 21:50Z read `PLAN ok`,
with the pipeline digest equal to the frozen one and 92 entries pending.
The meters at 21:50:39Z:

- NeuralWatt wet,
  with the whole 550 Wh of the free day that started at 2026-11-01T19:15:08Z and ends at 2026-11-02T19:15:08Z.
- Synthetic wet at 18.63 percent of its weekly allowance.
- DeepSeek wet with 9.34 USD;
  Xiaomi and Anthropic wet.
- Bedrock dry and left off;
  Hyper and OpenRouter dry.

The keyless cache audit read 14064 slice-cache records under 433 runs directories,
the newest written at 2026-11-01T07:42Z,
and none of the 8 cache versions set after it.

XingZ6025 on entry XingZ60 was not launched,
on the owner's word quoted at the head of this file.

## Round w410, stopped early for the reboot

Round w410 fixes the cross-vendor review's findings on round w409 and the second passes' findings on rounds w408 to
w409d.
Its tree is `/var/home/user/worktrees/tr-w410`,
head `840116e99ef3dcbdff92c0a8ecef04ab43fa96a8`,
nine commits on `2bc1ec8953`,
pushed to the remote branch `translation-repair-w410-wip` as a backup.
Nothing in it is gated yet.

Done:

- B1410:
  a re-ask keeps every provider its request excluded beside the one that answered,
  so a hinted JSON exchange is never re-asked on the provider it excluded
  (`otherThan` is a non-empty list).
- B1411:
  `routedJson`'s other-model re-ask with no reading is driven by a case.
- B1412:
  the three request builders check their literal against a seated request type,
  and the seat scan no longer takes an unrelated spread,
  a seat that may be absent or a bracketed call.
- B1413:
  the width scan reads fences inside list items and block quotes as CommonMark does,
  and counts columns from a table generated out of Unicode's `EastAsianWidth.txt` 17.0.0;
  its regex and the lint suppression for it are gone.

The newest commit is unfinished and fails:
a per-slice count of the repair lane's asks (B1414, planned) credits one slice with no ask,
and whether the count's attribution or the lane is at fault is not known.

Not started:
the seat tally's handling of a refusal raised after an exchange,
a stale comment in `archive/src/coverage-stage.ts`,
the smaller code points of the code reader (a promise chain in one test, an untagged logger,
a warning cooldown shared by two refusals),
the docs reader's findings (among them the living rule that still says a request naming no seat is not held,
the false lint claim of B1383,
and the stale width rule),
the six commit-message slips to record,
the round's ledger file and its index entries,
and the round's gates.
Ledger ids B1415 to B1434 are free for it.

## After the reboot

The lead's plan, revisable:

1.  Check that `tr-w410` is at `840116e99e` and clean;
    if the tree is lost,
    recreate it detached from `translation-repair-w410-wip`.
2.  Rerun the keyed pre-launch checks
    (the meters move),
    then launch XingZ6025 on entry XingZ60 on the frozen build `5722d602`,
    under the owner's earlier word "If you think we're launch ready just launch."
    NeuralWatt's free day runs to 2026-11-02T19:15:08Z.
3.  Resume round w410 with a fresh agent in its tree,
    from its brief and hand-back;
    then gate,
    rotation sweep,
    a cross-vendor review and second passes,
    and land.
    A landing while XingZ6025 runs means kill and relaunch on the new frozen build,
    by the rule in the family's
    [`configuration.md`](../../package/translation-repair/doc/configuration.md).

## Owner words of 2026-11-01

Typed messages and chosen option labels,
quoted exactly:

- "Move the pin to 0.5.0",
  then "Always move the pins up."
- "Keep exempt (Recommended)":
  measurements,
  the benches and probes that score a model on its provider,
  stay exempt from holds;
  corpus runs never ask a held model on its held provider.
- "Feel free to create remote branches."
- The reboot instruction quoted at the head of this file.

## Where the detail is

The lead's running notes,
the round briefs and hand-backs,
the landing,
comment,
rebuild and launch scripts,
and every gate and sweep log
are in the scratch folder `/var/home/user/temp/agent/tr-fan/`,
which a reboot may not keep:
the running notes in `w270-setup/handover-draft.md`,
round w410's brief and hand-back in `w410-fix/`,
the landing and comment files in `branch-only/`,
the rebuild and pre-launch logs in `frozen-next/`,
and the launcher in `s245/split/launch-rotation.ts`.
If that folder is lost,
this file,
the ledger,
the runbook
[`translation-repair-corpus-pass.md`](../runbook/translation-repair-corpus-pass.md)
and the branch `translation-repair-w410-wip` carry what the next session needs.
